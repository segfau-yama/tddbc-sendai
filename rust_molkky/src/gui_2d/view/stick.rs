use crate::core::Id;
use dioxus::prelude::*;

pub fn aim(origin: [f32; 2], target: Option<[f32; 2]>) -> Element {
    rsx! {
        if let Some([x, y]) = target {
            line { x1: "{origin[0]}", y1: "{origin[1]}", x2: "{x}", y2: "{y}",
                stroke: "#355fba", stroke_width: "0.008", stroke_dasharray: "0.03 0.02" }
            circle { cx: "{x}", cy: "{y}", r: "0.025", fill: "none", stroke: "#355fba", stroke_width: "0.008" }
        }
    }
}

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
