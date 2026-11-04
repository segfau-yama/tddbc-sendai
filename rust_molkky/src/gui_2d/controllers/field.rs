use super::{app::Phase, skittle, stick};
use crate::core::{Id, Molkky};
use crate::gui_2d::{
    models::{GuiField, GuiStick},
    services::GuiMolkky,
    view,
};
use dioxus::prelude::*;

/// view用の位置(m)と回転(rad)。未知のIdはNone。
pub fn pose(rules: &GuiMolkky, id: Id) -> Option<([f32; 2], f32)> {
    // ponytail: 通常12本なので走査する。多数の個体が必要になったら逆引きを追加する。
    let handle = if id == rules.stick_id {
        rules.stick_body
    } else {
        let (&collider, _) = rules
            .skittle_ids
            .iter()
            .find(|(_, skittle_id)| **skittle_id == id)?;
        rules.world.colliders[collider].parent()?
    };
    let body = &rules.world.bodies[handle];
    Some((body.translation().to_array(), body.rotation().angle()))
}

pub(super) fn render(
    field: &GuiField,
    stick: &GuiStick,
    rules: &GuiMolkky,
    phase: Phase,
) -> Element {
    let targets = if phase == Phase::Charging {
        rules.aim(stick, field.skittles())
    } else {
        Vec::new()
    };
    let skittles = field.skittles().iter().map(|pin| {
        let aimed = targets.iter().any(|target| target.id() == pin.id());
        skittle::render(pin, rules, aimed)
    });
    let scene = rsx! {
        {skittles}
        {view::aim(GuiField::THROW_POSITION, stick.target())}
        {stick::render(stick, rules, phase == Phase::Simulating)}
    };
    view::field(GuiField::BOUNDS, scene)
}
