use crate::core::SkittleState;
use crate::gui_2d::models::GuiSkittle;
use dioxus::prelude::*;

pub fn skittle(skittle: &GuiSkittle) -> Element {
    let [x, y] = skittle.position();
    let radius = GuiSkittle::RADIUS;
    let skittle = skittle.as_ref();
    let id = skittle.id();
    let number = skittle.number();
    let (state, fill) = match skittle.state() {
        SkittleState::Stand => ("stand", "#e5bd79"),
        SkittleState::Hit => ("hit", "#eaa63c"),
        SkittleState::Fall => ("fall", "#a5a5a5"),
    };
    let font_size = radius * 0.9;
    rsx! {
        g {
            id: "skittle-{id}",
            "data-state": state,
            transform: "translate({x} {y})",
            circle { r: "{radius}", fill, stroke: "#493b28", stroke_width: "0.01" }
            // フィールドのY反転を文字だけ打ち消す。
            text {
                transform: "scale(1 -1)",
                text_anchor: "middle",
                dominant_baseline: "central",
                font_size: "{font_size}",
                fill: "#241b10",
                "{number}"
            }
        }
    }
}
