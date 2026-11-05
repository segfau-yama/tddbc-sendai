use super::GuiMolkky;
use crate::core::{Id, Molkky, SkittleState, StickState};
use crate::gui_2d::models::{GuiSkittle, GuiStick};
use rapier2d::prelude::*;

pub const TIME_STEP: f32 = 1.0 / 60.0;
pub(super) const DAMPING: f32 = 2.0;

/// 共有ワールドを一度進める。接触と停止の状態はhit/chain_hitを適用する。
pub fn step(
    rules: &mut GuiMolkky,
    skittles: &[GuiSkittle],
    stick: GuiStick,
) -> (Vec<GuiSkittle>, GuiStick) {
    if stick.as_ref().id() != rules.stick_id {
        return (skittles.to_vec(), stick);
    }
    let body = &mut rules.world.bodies[rules.stick_body];
    if stick.as_ref().state() == StickState::Held {
        body.set_translation(Vector::from_array(stick.position()), false);
        body.set_rotation(Rotation::from_angle(stick.angle()), false);
        body.set_linvel(Vector::ZERO, false);
        body.set_angvel(0.0, false);
        body.sleep();
        return (skittles.to_vec(), stick);
    }
    // moveが用意した初速を一度反映し、以降は物理計算の結果を使う。
    if stick.as_ref().state() == StickState::Move
        && (body.translation().to_array() != stick.position()
            || body.linvel().to_array() != stick.velocity()
            || body.rotation().angle() != stick.angle()
            || body.angvel() != stick.angular_velocity())
    {
        body.set_translation(Vector::from_array(stick.position()), false);
        body.set_rotation(Rotation::from_angle(stick.angle()), false);
        body.set_linvel(Vector::from_array(stick.velocity()), false);
        body.set_angvel(stick.angular_velocity(), false);
        body.wake_up(true);
    }
    // 棒と全スキットルが同じ時刻の衝突結果を使う。
    rules.world.step();
    for pair in rules.world.narrow_phase.contact_pairs() {
        let handles = [pair.collider1, pair.collider2];
        if handles.iter().all(|handle| {
            rules.world.colliders[*handle]
                .parent()
                .is_none_or(|body| rules.world.bodies[body].is_sleeping())
        }) {
            continue;
        }
        let impulse = pair.total_impulse_magnitude();
        for key in [(handles[0], handles[1]), (handles[1], handles[0])] {
            let peak = rules.impacts.entry(key).or_default();
            *peak = peak.max(impulse);
        }
    }
    let body = &rules.world.bodies[rules.stick_body];
    let updated = stick.update(
        body.translation().to_array(),
        body.rotation().angle(),
        if body.is_sleeping() {
            [0.0; 2]
        } else {
            body.linvel().to_array()
        },
        if body.is_sleeping() {
            0.0
        } else {
            body.angvel()
        },
    );
    let mut skittles: Vec<_> = skittles
        .iter()
        .map(|skittle| {
            let Some((&collider, _)) = rules
                .skittle_ids
                .iter()
                .find(|(_, id)| **id == skittle.as_ref().id())
            else {
                return *skittle;
            };
            let body = &rules.world.bodies[rules.world.colliders[collider].parent().unwrap()];
            skittle.update(
                body.translation().to_array(),
                body.rotation().angle(),
                skittle.as_ref().state(),
                false,
            )
        })
        .collect();
    let contacts = contacts(rules);
    // 接触履歴のないHitで再開した場合も、停止時に既存のhitで確定する。
    for skittle in skittles.iter_mut().filter(|s| {
        s.as_ref().state() == SkittleState::Hit
            && !contacts
                .iter()
                .any(|&(a, b)| a == s.as_ref().id() || b == s.as_ref().id())
    }) {
        *skittle = rules.hit(&updated, skittle);
    }
    for (a, b) in contacts {
        let stick_id = updated.as_ref().id();
        if a == stick_id || b == stick_id {
            let id = if a == stick_id { b } else { a };
            if let Some(skittle) = skittles.iter_mut().find(|s| s.as_ref().id() == id) {
                *skittle = rules.hit(&updated, skittle);
            }
        } else if let (Some(left), Some(right)) = (
            skittles.iter().position(|s| s.as_ref().id() == a),
            skittles.iter().position(|s| s.as_ref().id() == b),
        ) {
            let first = skittles[left];
            let second = skittles[right];
            skittles[left] = rules.chain_hit(&second, &first);
            skittles[right] = rules.chain_hit(&first, &second);
        }
    }
    (skittles, updated)
}

/// 一投で実際に力積が発生した個体の組。停止時の再評価まで履歴を保持する。
pub fn contacts(rules: &GuiMolkky) -> Vec<(Id, Id)> {
    rules
        .impacts
        .iter()
        .filter_map(|(&(a, b), &impulse)| {
            if impulse <= 0.0 || a.into_raw_parts() >= b.into_raw_parts() {
                return None;
            }
            let id = |handle| {
                if handle == rules.stick_collider {
                    Some(rules.stick_id)
                } else {
                    rules.skittle_ids.get(&handle).copied()
                }
            };
            Some((id(a)?, id(b)?))
        })
        .collect()
}
