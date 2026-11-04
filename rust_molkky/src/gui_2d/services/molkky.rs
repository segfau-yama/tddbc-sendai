use crate::core::{Id, Molkky, Skittle, SkittleState};
use crate::gui_2d::models::{GuiField, GuiStick};
use rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct GuiMolkky {
    world: PhysicsWorld,
    stick_id: Id,
    stick_body: RigidBodyHandle,
    skittle_ids: HashMap<ColliderHandle, Id>,
    peak_impulses: HashMap<Id, f32>,
    origin: [f32; 2],
    active: bool,
}

impl GuiMolkky {
    pub const TIME_STEP: f32 = 1.0 / 60.0;
    pub const SKITTLE_RADIUS: f32 = 0.0275;
    pub const STICK_RADIUS: f32 = 0.0275;
    pub const STICK_LENGTH: f32 = 0.225;
    pub const SKITTLE_MASS: f32 = 0.15;
    pub const STICK_MASS: f32 = 0.25;
    pub const DAMPING: f32 = 2.0;
    /// N·s。高さ方向の転倒を近似するための調整値。
    pub const FALL_IMPULSE: f32 = 0.08;

    /// originとplacementsはメートル座標。標準配置の生成は呼び出し側で行う。
    pub fn new(stick: &GuiStick, origin: [f32; 2], placements: &[(Id, [f32; 2])]) -> Self {
        assert!(origin.iter().all(|v| v.is_finite()), "投擲位置は有限値です");
        let mut world = PhysicsWorld::new();
        world.gravity = Vector::ZERO;
        world.integration_parameters.dt = Self::TIME_STEP;
        let [mut x, mut y, mut width, mut height] = GuiField::BOUNDS;
        x += GuiField::WALL_THICKNESS;
        y += GuiField::WALL_THICKNESS;
        width -= 2.0 * GuiField::WALL_THICKNESS;
        height -= 2.0 * GuiField::WALL_THICKNESS;
        let thickness = 0.1;
        // 描画する外周の壁に内側を合わせ、反射はRapierへ任せる。
        for (center, half_size) in [
            (
                [x - thickness, y + height / 2.0],
                [thickness, height / 2.0 + thickness],
            ),
            (
                [x + width + thickness, y + height / 2.0],
                [thickness, height / 2.0 + thickness],
            ),
            (
                [x + width / 2.0, y - thickness],
                [width / 2.0 + thickness, thickness],
            ),
            (
                [x + width / 2.0, y + height + thickness],
                [width / 2.0 + thickness, thickness],
            ),
        ] {
            world.insert_collider(
                ColliderBuilder::cuboid(half_size[0], half_size[1])
                    .translation(Vector::from_array(center))
                    .restitution(0.5)
                    .restitution_combine_rule(CoefficientCombineRule::Max)
                    .friction(0.0)
                    .friction_combine_rule(CoefficientCombineRule::Min),
                None,
            );
        }
        let (stick_body, _) = world.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::from_array(origin))
                .linear_damping(Self::DAMPING)
                .angular_damping(Self::DAMPING)
                .ccd_enabled(true)
                .sleeping(true),
            ColliderBuilder::capsule_y(
                Self::STICK_LENGTH / 2.0 - Self::STICK_RADIUS,
                Self::STICK_RADIUS,
            )
            .mass(Self::STICK_MASS),
        );
        let mut ids = HashSet::from([stick.stick().id()]);
        let mut skittle_ids = HashMap::new();
        for &(id, position) in placements {
            assert!(ids.insert(id), "個体Idは一意です");
            assert!(position.iter().all(|v| v.is_finite()), "配置は有限値です");
            let (_, collider) = world.insert(
                RigidBodyBuilder::dynamic()
                    .translation(Vector::from_array(position))
                    .linear_damping(Self::DAMPING)
                    .angular_damping(Self::DAMPING)
                    .ccd_enabled(true)
                    .sleeping(true),
                ColliderBuilder::ball(Self::SKITTLE_RADIUS).mass(Self::SKITTLE_MASS),
            );
            skittle_ids.insert(collider, id);
        }
        Self {
            world,
            stick_id: stick.stick().id(),
            stick_body,
            skittle_ids,
            peak_impulses: HashMap::new(),
            origin,
            active: false,
        }
    }

    /// 1tickだけ進める。描画や実時間の待機は行わない。
    pub fn step(&mut self) {
        if !self.active {
            return;
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
        self.active = self
            .world
            .bodies
            .iter()
            .any(|(_, body)| !body.is_sleeping());
    }

    pub fn is_settled(&self) -> bool {
        !self.active
    }

    /// view用の位置(m)と回転(rad)。未知のIdはNone。
    pub fn pose(&self, id: Id) -> Option<([f32; 2], f32)> {
        // ponytail: 通常12本なので走査する。多数の個体が必要になったら逆引きを追加する。
        let handle = if id == self.stick_id {
            self.stick_body
        } else {
            let (&collider, _) = self
                .skittle_ids
                .iter()
                .find(|(_, skittle_id)| **skittle_id == id)?;
            self.world.colliders[collider].parent()?
        };
        let body = &self.world.bodies[handle];
        Some((body.translation().to_array(), body.rotation().angle()))
    }

    /// 位置とIdを保って立て直す。core個体のStandへの反映は呼び出し側で行う。
    pub fn reset_throw(&mut self) {
        for &collider in self.skittle_ids.keys() {
            let body = &mut self.world.bodies[self.world.colliders[collider].parent().unwrap()];
            body.set_rotation(Rotation::IDENTITY, false);
            body.set_linvel(Vector::ZERO, false);
            body.set_angvel(0.0, false);
            body.sleep();
        }
        self.peak_impulses.clear();
        let body = &mut self.world.bodies[self.stick_body];
        body.set_translation(Vector::from_array(self.origin), false);
        body.set_rotation(Rotation::IDENTITY, false);
        body.set_linvel(Vector::ZERO, false);
        body.set_angvel(0.0, false);
        body.sleep();
        self.active = false;
    }

    fn resolve_hit(&self, skittle: &Skittle) -> Skittle {
        if skittle.state() == SkittleState::Stand
            && self
                .peak_impulses
                .get(&skittle.id())
                .is_some_and(|&peak| peak >= Self::FALL_IMPULSE)
        {
            skittle.with_state(SkittleState::Fall)
        } else {
            *skittle
        }
    }

    fn direction(&self, stick: &GuiStick) -> Option<Vector> {
        if stick.stick().id() != self.stick_id {
            return None;
        }
        let target = Vector::from_array(stick.target()?);
        (target - Vector::from_array(self.origin)).try_normalize()
    }
}

impl Molkky for GuiMolkky {
    type Stick = GuiStick;
    type Skittle = Skittle;
    type ThrowOutput = bool;

    fn aim(&self, stick: &GuiStick, skittles: &[Self::Skittle]) -> Vec<Self::Skittle> {
        let Some(direction) = self.direction(stick) else {
            return Vec::new();
        };
        skittles
            .iter()
            .filter(|skittle| {
                let Some((pos, _)) = self.pose(skittle.id()) else {
                    return false;
                };
                let relative = Vector::from_array(pos) - Vector::from_array(self.origin);
                let along = relative.dot(direction);
                let across = (relative - direction * along).length();
                skittle.state() == SkittleState::Stand
                    && along >= 0.0
                    && across <= Self::SKITTLE_RADIUS + Self::STICK_RADIUS
            })
            .copied()
            .collect()
    }

    fn throw(&mut self, stick: &GuiStick) -> bool {
        let Some(direction) = self.direction(stick) else {
            return false;
        };
        if self.active || !stick.speed().is_finite() || stick.speed() <= 0.0 {
            return false;
        }
        self.reset_throw();
        let body = &mut self.world.bodies[self.stick_body];
        body.set_rotation(
            Rotation::from_angle(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2),
            true,
        );
        body.set_linvel(direction * stick.speed(), true);
        self.active = true;
        true
    }

    fn hit(&self, stick: &GuiStick, skittle: &Self::Skittle) -> Self::Skittle {
        if stick.stick().id() == self.stick_id {
            self.resolve_hit(skittle)
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
            self.resolve_hit(chained_skittle)
        } else {
            *chained_skittle
        }
    }
}
