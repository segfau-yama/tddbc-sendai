use super::{GuiMolkky, TIME_STEP, molkky::DAMPING};
use crate::{
    Field, Skittle, SkittleState,
    gui_2d::models::{GuiField, GuiSkittle, GuiStick},
};
use rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};

/// 全チーム共通のフィールド、棒、標準配置のスキットルを用意する。
pub fn create_field() -> (GuiField, GuiStick, Vec<GuiSkittle>) {
    let rows: &[&[i32]] = &[&[1, 2], &[3, 10, 4], &[5, 11, 12, 6], &[7, 9, 8]];
    let mut skittles: Vec<_> = rows
        .iter()
        .enumerate()
        .flat_map(|(row, numbers)| {
            numbers.iter().enumerate().map(move |(column, &number)| {
                let radius = GuiSkittle::RADIUS;
                let x = (2.0 * column as f32 - (numbers.len() - 1) as f32) * radius;
                GuiSkittle::new(
                    Skittle::new(number),
                    [x, 1.0 + row as f32 * 3.0_f32.sqrt() * radius],
                )
            })
        })
        .collect();
    skittles.sort_by_key(|skittle| skittle.as_ref().number());
    let field = GuiField::new(Field::new());
    let stick = GuiStick::default();
    (field, stick, skittles)
}

/// 各モデルの位置(m)と姿勢から、全チーム共通の物理ワールドを構築する。
pub fn create_world(stick: &GuiStick, skittles: &[GuiSkittle]) -> GuiMolkky {
    let mut world = PhysicsWorld::new();
    world.gravity = Vector::ZERO;
    world.integration_parameters.dt = TIME_STEP;
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
    let (stick_body, stick_collider) = world.insert(
        RigidBodyBuilder::dynamic()
            .translation(Vector::from_array(stick.position()))
            .linear_damping(DAMPING)
            .angular_damping(DAMPING)
            .ccd_enabled(true)
            .sleeping(true),
        ColliderBuilder::capsule_y(GuiStick::LENGTH / 2.0 - GuiStick::RADIUS, GuiStick::RADIUS)
            .mass(GuiStick::MASS),
    );
    let mut ids = HashSet::from([stick.stick().id()]);
    let mut skittle_ids = HashMap::new();
    for skittle in skittles {
        let id = skittle.as_ref().id();
        let position = skittle.position();
        assert!(ids.insert(id), "個体Idは一意です");
        assert!(position.iter().all(|v| v.is_finite()), "配置は有限値です");
        let (_, collider) = world.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::from_array(position))
                .linear_damping(DAMPING)
                .angular_damping(DAMPING)
                .ccd_enabled(true)
                .sleeping(skittle.as_ref().state() != SkittleState::Hit),
            ColliderBuilder::ball(GuiSkittle::RADIUS).mass(GuiSkittle::MASS),
        );
        skittle_ids.insert(collider, id);
    }
    GuiMolkky {
        world,
        stick_id: stick.stick().id(),
        stick_body,
        skittle_ids,
        stick_collider,
        impacts: HashMap::new(),
    }
}
