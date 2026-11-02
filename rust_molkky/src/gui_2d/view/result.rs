use crate::core::Team;
use dioxus::prelude::*;

/// 終了と勝者の判定は呼び出し側で行う。
pub fn result(winner: Option<&Team>) -> Element {
    let message = match winner {
        Some(team) => format!("{}の勝利！", team.name()),
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
