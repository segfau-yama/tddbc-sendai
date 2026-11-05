use dioxus::prelude::*;

/// 終了と勝者の判定は呼び出し側で行う。
#[component]
pub fn Result(winner: Option<String>) -> Element {
    let message = match winner {
        Some(name) => format!("{name}の勝利！"),
        None => "勝者なし".to_owned(),
    };
    rsx! {
        section {
            aria_label: "試合結果",
            h2 { "試合結果" }
            p { "{message}" }
        }
    }
}
