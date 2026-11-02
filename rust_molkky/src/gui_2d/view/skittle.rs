use crate::core::{Skittle, SkittleState};
use dioxus::prelude::*;

pub fn skittle(pin: &Skittle, position: [f32; 2], radius: f32) -> Element {
    let [x, y] = position;
    let id = pin.id();
    let number = pin.number();
    let (state, fill) = match pin.state() {
        SkittleState::Stand => ("stand", "#e5bd79"),
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
