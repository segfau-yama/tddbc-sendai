use crate::core::{Id, Molkky, SkittleState};
use crate::gui_2d::models::{GuiField, GuiSkittle, GuiStick};
use rapier2d::prelude::*;
use std::collections::HashMap;

#[cfg(test)]
#[path = "../../../tests/gui_2d/services/molkky.rs"]
mod tests;

/// N·s。高さ方向の転倒を近似する調整値。
const FALL_IMPULSE: f32 = 0.08;

pub struct GuiMolkky {
    pub(crate) world: PhysicsWorld,
    pub(crate) stick_body: RigidBodyHandle,
    pub(crate) stick_collider: ColliderHandle,
    pub(crate) skittle_ids: HashMap<ColliderHandle, Id>,
    pub(crate) impacts: HashMap<(ColliderHandle, ColliderHandle), f32>,
}

impl Molkky for GuiMolkky {
    type Stick = GuiStick;
    type Skittle = GuiSkittle;

    fn aim(&self, stick: &GuiStick, skittles: &[GuiSkittle]) -> Vec<GuiSkittle> {
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
            .filter_map(|skittle| {
                if skittle.as_ref().state() != SkittleState::Stand {
                    return None;
                }
                let (&collider, _) = self
                    .skittle_ids
                    .iter()
                    .find(|(_, id)| **id == skittle.as_ref().id())?;
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
                Some(skittle.update(
                    body.translation().to_array(),
                    body.rotation().angle(),
                    skittle.as_ref().state(),
                    true,
                ))
            })
            .collect()
    }

    fn hit(&self, _stick: &GuiStick, skittle: &GuiSkittle) -> GuiSkittle {
        let Some((&collider, _)) = self
            .skittle_ids
            .iter()
            .find(|(_, id)| **id == skittle.as_ref().id())
        else {
            return *skittle;
        };
        let direct = self
            .impacts
            .get(&(self.stick_collider, collider))
            .is_some_and(|&impulse| impulse > 0.0);
        hit_result(self, collider, skittle, direct)
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
        hit_result(self, target_collider, target, true)
    }
}

fn hit_result(
    rules: &GuiMolkky,
    collider: ColliderHandle,
    skittle: &GuiSkittle,
    contacted: bool,
) -> GuiSkittle {
    let body = &rules.world.bodies[rules.world.colliders[collider].parent().unwrap()];
    let state = if !contacted && skittle.as_ref().state() == SkittleState::Stand {
        SkittleState::Stand
    } else if !body.is_sleeping() {
        SkittleState::Hit
    } else if skittle.as_ref().state() == SkittleState::Fall
        || rules
            .impacts
            .iter()
            .any(|((_, target), &peak)| *target == collider && peak >= FALL_IMPULSE)
    {
        SkittleState::Fall
    } else {
        SkittleState::Stand
    };
    skittle.update(
        body.translation().to_array(),
        body.rotation().angle(),
        state,
        skittle.aimed(),
    )
}
