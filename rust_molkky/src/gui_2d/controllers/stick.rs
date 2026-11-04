use super::field::pose;
use crate::gui_2d::{
    models::{GuiField, GuiStick},
    services::GuiMolkky,
    view,
};
use dioxus::prelude::*;

#[cfg(test)]
#[path = "../../../tests/gui_2d/controllers/stick.rs"]
mod tests;

pub(super) fn render(stick: &GuiStick, rules: &GuiMolkky, simulating: bool) -> Element {
    let (position, angle) = if simulating && rules.flying {
        pose(rules, stick.stick().id()).unwrap()
    } else {
        (GuiField::THROW_POSITION, 0.0)
    };
    view::stick(
        stick.stick().id(),
        position,
        angle,
        GuiStick::LENGTH,
        2.0 * GuiStick::RADIUS,
    )
}
