use dioxus::prelude::*;

/// 速度と正の上限はm/sで受け取る。時間変化は呼び出し側で計算する。
pub fn speedometer(speed: f32, max_speed: f32) -> Element {
    rsx! {
        div {
            meter {
                aria_label: "投擲速度",
                min: "0", max: "{max_speed}", value: "{speed}",
                "{speed:.1} m/s"
            }
            span { " {speed:.1} m/s" }
        }
    }
}
