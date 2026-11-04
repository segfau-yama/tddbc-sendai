use crate::core::{Id, Molkky, Move, SkittleState, StickState};
use crate::gui_2d::models::{GuiField, GuiSkittle, GuiStick};
use rapier2d::prelude::*;
use std::collections::HashMap;

pub const TIME_STEP: f32 = 1.0 / 60.0;
pub(super) const DAMPING: f32 = 2.0;
/// N·s。高さ方向の転倒を近似する調整値。
const FALL_IMPULSE: f32 = 0.08;

pub struct GuiMolkky {
    pub(super) world: PhysicsWorld,
    pub(super) stick_id: Id,
    pub(super) stick_body: RigidBodyHandle,
    pub(super) stick_collider: ColliderHandle,
    pub(super) skittle_ids: HashMap<ColliderHandle, Id>,
    pub(super) impacts: HashMap<(ColliderHandle, ColliderHandle), f32>,
}

impl GuiMolkky {
    /// 保持中は照準だけ、投擲中は共有ワールドを1刻み更新してモデルを返す。
    pub fn update(&mut self, field: &GuiField, stick: GuiStick) -> (GuiField, GuiStick) {
        if stick.as_ref().id() != self.stick_id {
            return (field.clone(), stick);
        }
        let body = &mut self.world.bodies[self.stick_body];
        if stick.as_ref().state() == StickState::Held {
            body.set_translation(Vector::from_array(stick.position), false);
            body.set_rotation(Rotation::from_angle(stick.angle), false);
            body.set_linvel(Vector::ZERO, false);
            body.set_angvel(0.0, false);
            body.sleep();
            let targets = self.aim(&stick, field.skittles());
            let pins = field
                .skittles()
                .iter()
                .map(|pin| GuiSkittle {
                    aimed: targets
                        .iter()
                        .any(|target| target.as_ref().id() == pin.as_ref().id()),
                    ..*pin
                })
                .collect();
            return (GuiField::new(*field.as_ref(), pins), stick);
        }
        // throwが作った初速だけを反映する。更新済みモデルはボディと一致している。
        if stick.as_ref().state() == StickState::Move
            && (body.translation().to_array() != stick.position
                || body.linvel().to_array() != stick.velocity
                || body.rotation().angle() != stick.angle
                || body.angvel() != stick.angular_velocity)
        {
            body.set_translation(Vector::from_array(stick.position), false);
            body.set_rotation(Rotation::from_angle(stick.angle), false);
            body.set_linvel(Vector::from_array(stick.velocity), false);
            body.set_angvel(stick.angular_velocity, false);
            body.wake_up(true);
        }
        self.world.step();
        for pair in self.world.narrow_phase.contact_pairs() {
            let handles = [pair.collider1, pair.collider2];
            if handles.iter().all(|h| {
                self.world.colliders[*h]
                    .parent()
                    .is_none_or(|body| self.world.bodies[body].is_sleeping())
            }) {
                continue;
            }
            let impulse = pair.total_impulse_magnitude();
            for key in [(handles[0], handles[1]), (handles[1], handles[0])] {
                let peak = self.impacts.entry(key).or_default();
                *peak = peak.max(impulse);
            }
        }
        let body = &self.world.bodies[self.stick_body];
        let velocity = if body.is_sleeping() {
            [0.0; 2]
        } else {
            body.linvel().to_array()
        };
        let updated = GuiStick::r#move(GuiStick {
            position: body.translation().to_array(),
            angle: body.rotation().angle(),
            velocity,
            angular_velocity: if body.is_sleeping() {
                0.0
            } else {
                body.angvel()
            },
            speed: velocity[0].hypot(velocity[1]),
            ..stick
        });
        let pins = field
            .skittles()
            .iter()
            .map(|pin| {
                let mut result = self.hit(&updated, pin);
                for source in field.skittles() {
                    if source.as_ref().id() != pin.as_ref().id() {
                        result = self.chain_hit(source, &result);
                    }
                }
                result.aimed = false;
                result
            })
            .collect();
        (GuiField::new(*field.as_ref(), pins), updated)
    }
}

impl Molkky for GuiMolkky {
    type Stick = GuiStick;
    type Skittle = GuiSkittle;

    fn aim(&self, stick: &GuiStick, skittles: &[GuiSkittle]) -> Vec<GuiSkittle> {
        if stick.as_ref().id() != self.stick_id {
            return Vec::new();
        }
        let Some(target) = stick.target() else {
            return Vec::new();
        };
        let Some(direction) =
            (Vector::from_array(target) - Vector::from_array(stick.position())).try_normalize()
        else {
            return Vec::new();
        };
        let length = GuiField::BOUNDS[2].hypot(GuiField::BOUNDS[3]);
        let corridor = Cuboid::new(Vector::new(GuiStick::RADIUS, length / 2.0));
        let pose = Pose::from_parts(
            Vector::from_array(stick.position()) + direction * (length / 2.0),
            Rotation::from_angle(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2),
        );
        skittles
            .iter()
            .filter_map(|pin| {
                if pin.as_ref().state() != SkittleState::Stand {
                    return None;
                }
                let (&collider, _) = self
                    .skittle_ids
                    .iter()
                    .find(|(_, id)| **id == pin.as_ref().id())?;
                let body = &self.world.bodies[self.world.colliders[collider].parent()?];
                let relative = pose.inv_mul(body.position());
                if !self
                    .world
                    .narrow_phase
                    .query_dispatcher()
                    .intersection_test(&relative, &corridor, self.world.colliders[collider].shape())
                    .is_ok_and(|hit| hit.intersecting)
                {
                    return None;
                }
                Some(GuiSkittle {
                    position: body.translation().to_array(),
                    angle: body.rotation().angle(),
                    aimed: true,
                    ..*pin
                })
            })
            .collect()
    }

    fn hit(&self, stick: &GuiStick, skittle: &GuiSkittle) -> GuiSkittle {
        if stick.as_ref().id() != self.stick_id {
            return *skittle;
        }
        let Some((&collider, _)) = self
            .skittle_ids
            .iter()
            .find(|(_, id)| **id == skittle.as_ref().id())
        else {
            return *skittle;
        };
        let body = &self.world.bodies[self.world.colliders[collider].parent().unwrap()];
        let direct = self
            .impacts
            .get(&(self.stick_collider, collider))
            .is_some_and(|&impulse| impulse > 0.0);
        let state = if direct
            || matches!(
                skittle.as_ref().state(),
                SkittleState::Hit | SkittleState::Fall
            ) {
            if !body.is_sleeping() {
                SkittleState::Hit
            } else if skittle.as_ref().state() == SkittleState::Fall
                || self
                    .impacts
                    .iter()
                    .filter(|((_, target), _)| *target == collider)
                    .any(|(_, &peak)| peak >= FALL_IMPULSE)
            {
                SkittleState::Fall
            } else {
                SkittleState::Stand
            }
        } else {
            skittle.as_ref().state()
        };
        GuiSkittle {
            skittle: skittle.as_ref().with_state(state),
            position: body.translation().to_array(),
            angle: body.rotation().angle(),
            ..*skittle
        }
    }

    fn chain_hit(&self, source: &GuiSkittle, target: &GuiSkittle) -> GuiSkittle {
        let Some((&source_collider, _)) = self
            .skittle_ids
            .iter()
            .find(|(_, id)| **id == source.as_ref().id())
        else {
            return *target;
        };
        let Some((&target_collider, _)) = self
            .skittle_ids
            .iter()
            .find(|(_, id)| **id == target.as_ref().id())
        else {
            return *target;
        };
        if source_collider == target_collider
            || !self
                .impacts
                .get(&(source_collider, target_collider))
                .is_some_and(|&peak| peak > 0.0)
        {
            return *target;
        }
        let body = &self.world.bodies[self.world.colliders[target_collider].parent().unwrap()];
        let state = if !body.is_sleeping() {
            SkittleState::Hit
        } else if target.as_ref().state() == SkittleState::Fall
            || self
                .impacts
                .iter()
                .filter(|((_, target), _)| *target == target_collider)
                .any(|(_, &peak)| peak >= FALL_IMPULSE)
        {
            SkittleState::Fall
        } else {
            SkittleState::Stand
        };
        GuiSkittle {
            skittle: target.as_ref().with_state(state),
            position: body.translation().to_array(),
            angle: body.rotation().angle(),
            ..*target
        }
    }

    fn stand_up(&mut self, skittle: &GuiSkittle) -> GuiSkittle {
        if let Some((&collider, _)) = self
            .skittle_ids
            .iter()
            .find(|(_, id)| **id == skittle.as_ref().id())
        {
            let body = &mut self.world.bodies[self.world.colliders[collider].parent().unwrap()];
            body.set_rotation(Rotation::IDENTITY, false);
            body.set_linvel(Vector::ZERO, false);
            body.set_angvel(0.0, false);
            body.sleep();
            self.impacts
                .retain(|&(source, target), _| source != collider && target != collider);
        }
        GuiSkittle {
            skittle: skittle.as_ref().with_state(SkittleState::Stand),
            angle: 0.0,
            aimed: false,
            ..*skittle
        }
    }
}
