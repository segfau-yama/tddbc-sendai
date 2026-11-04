use crate::core::{Id, Molkky, Skittle, SkittleState};
use crate::gui_2d::models::{GuiSkittle, GuiStick};
use rapier2d::prelude::*;
use std::collections::HashMap;

pub const TIME_STEP: f32 = 1.0 / 60.0;
pub(super) const DAMPING: f32 = 2.0;
/// N·s。高さ方向の転倒を近似するための調整値。
const FALL_IMPULSE: f32 = 0.08;

pub struct GuiMolkky {
    pub(in crate::gui_2d) world: PhysicsWorld,
    pub(in crate::gui_2d) stick_id: Id,
    pub(in crate::gui_2d) stick_body: RigidBodyHandle,
    pub(in crate::gui_2d) skittle_ids: HashMap<ColliderHandle, Id>,
    pub(in crate::gui_2d) peak_impulses: HashMap<Id, f32>,
    pub(in crate::gui_2d) origin: [f32; 2],
    pub(in crate::gui_2d) flying: bool,
}

impl Molkky for GuiMolkky {
    type Stick = GuiStick;
    type Skittle = Skittle;

    /// 完了後の呼び出しは次投を始める。開始条件が無効ならワールドを変更しない。
    fn fly(&mut self, stick: &GuiStick) -> bool {
        if !self.flying {
            let Some(direction) = direction(self, stick) else {
                return false;
            };
            if !stick.speed().is_finite() || stick.speed() <= 0.0 {
                return false;
            }
            let body = &mut self.world.bodies[self.stick_body];
            body.set_translation(Vector::from_array(self.origin), false);
            body.set_rotation(
                Rotation::from_angle(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2),
                false,
            );
            body.set_angvel(0.0, false);
            body.set_linvel(direction * stick.speed(), true);
        }
        self.world.step();
        for pair in self.world.narrow_phase.contact_pairs() {
            // 立て直したsleep中のピン同士の古い接触記録は使わない。
            let handles = [pair.collider1, pair.collider2];
            let sleeping = handles.iter().all(|h| {
                self.world.colliders[*h]
                    .parent()
                    .is_none_or(|body| self.world.bodies[body].is_sleeping())
            });
            if sleeping {
                continue;
            }
            let impulse = pair.total_impulse_magnitude();
            for handle in handles {
                if let Some(&id) = self.skittle_ids.get(&handle) {
                    let peak = self.peak_impulses.entry(id).or_default();
                    *peak = peak.max(impulse);
                }
            }
        }
        // 棒が先に止まっても、ピンの連鎖が終わるまで採点を待つ。
        self.flying = self
            .world
            .bodies
            .iter()
            .any(|(_, body)| !body.is_sleeping());
        self.flying
    }

    /// 倒れた地点で対象個体だけを立て直し、今回の接触記録も取り除く。
    fn stand_up(&mut self, skittle: &Self::Skittle) -> Self::Skittle {
        if let Some((&collider, _)) = self.skittle_ids.iter().find(|(_, id)| **id == skittle.id()) {
            let body = &mut self.world.bodies[self.world.colliders[collider].parent().unwrap()];
            body.set_rotation(Rotation::IDENTITY, false);
            body.set_linvel(Vector::ZERO, false);
            body.set_angvel(0.0, false);
            body.sleep();
        }
        self.peak_impulses.remove(&skittle.id());
        skittle.with_state(SkittleState::Stand)
    }

    fn aim(&self, stick: &GuiStick, skittles: &[Self::Skittle]) -> Vec<Self::Skittle> {
        let Some(direction) = direction(self, stick) else {
            return Vec::new();
        };
        skittles
            .iter()
            .filter(|skittle| {
                let Some((&collider, _)) =
                    self.skittle_ids.iter().find(|(_, id)| **id == skittle.id())
                else {
                    return false;
                };
                let body = self.world.colliders[collider].parent().unwrap();
                let relative =
                    self.world.bodies[body].translation() - Vector::from_array(self.origin);
                let along = relative.dot(direction);
                let across = (relative - direction * along).length();
                skittle.state() == SkittleState::Stand
                    && along >= 0.0
                    && across <= GuiSkittle::RADIUS + GuiStick::RADIUS
            })
            .copied()
            .collect()
    }

    fn hit(&self, stick: &GuiStick, skittle: &Self::Skittle) -> Self::Skittle {
        if stick.stick().id() == self.stick_id {
            resolve_hit(self, skittle)
        } else {
            *skittle
        }
    }

    fn chain_hit(
        &self,
        fallen_skittle: &Self::Skittle,
        chained_skittle: &Self::Skittle,
    ) -> Self::Skittle {
        if fallen_skittle.state() == SkittleState::Fall {
            resolve_hit(self, chained_skittle)
        } else {
            *chained_skittle
        }
    }
}

fn resolve_hit(rules: &GuiMolkky, skittle: &Skittle) -> Skittle {
    if skittle.state() == SkittleState::Stand
        && rules
            .peak_impulses
            .get(&skittle.id())
            .is_some_and(|&peak| peak >= FALL_IMPULSE)
    {
        skittle.with_state(SkittleState::Fall)
    } else {
        *skittle
    }
}

fn direction(rules: &GuiMolkky, stick: &GuiStick) -> Option<Vector> {
    if stick.stick().id() != rules.stick_id {
        return None;
    }
    let target = Vector::from_array(stick.target()?);
    (target - Vector::from_array(rules.origin)).try_normalize()
}
