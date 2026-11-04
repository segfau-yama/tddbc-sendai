use super::field::pose;
use crate::core::Skittle;
use crate::gui_2d::{models::GuiSkittle, services::GuiMolkky, view};
use dioxus::prelude::*;

pub(super) fn render(skittle: &Skittle, rules: &GuiMolkky, aimed: bool) -> Element {
    let position = pose(rules, skittle.id()).unwrap().0;
    let id = skittle.id();
    let class = if aimed { "aimed" } else { "" };
    rsx! { g { key: "{id}", class, {view::skittle(skittle, position, GuiSkittle::RADIUS)} } }
}
