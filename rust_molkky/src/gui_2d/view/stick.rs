use crate::core::Id;
use dioxus::prelude::*;

/// angleはworld座標のラジアン。棒の長軸はローカルY軸。
pub fn stick(id: Id, position: [f32; 2], angle: f32, length: f32, width: f32) -> Element {
    let [x, y] = position;
    let degrees = angle.to_degrees();
    let left = -width / 2.0;
    let top = -length / 2.0;
    let radius = width / 2.0;
    rsx! {
        g {
            id: "stick-{id}",
            transform: "translate({x} {y}) rotate({degrees})",
            rect {
                x: "{left}", y: "{top}",
                width: "{width}", height: "{length}", rx: "{radius}",
                fill: "#995c32", stroke: "#493b28", stroke_width: "0.01"
            }
        }
    }
}
