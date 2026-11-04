use crate::core::StickState;
use crate::gui_2d::{
    models::{GuiField, GuiStick},
    view,
};
use dioxus::prelude::*;

pub(super) fn render(field: &GuiField, stick: &GuiStick) -> Element {
    let aiming = stick.as_ref().state() == StickState::Held && stick.target().is_some();
    let skittles = field.skittles().iter().map(|pin| {
        let class = if aiming && pin.aimed() { "aimed" } else { "" };
        let id = pin.as_ref().id();
        rsx! { g { key: "{id}", class, {view::skittle(pin)} } }
    });
    let scene = rsx! {
        {skittles}
        if aiming { {view::aim(stick.position(), stick.target())} }
        {view::stick(stick)}
    };
    view::field(GuiField::BOUNDS, scene)
}
