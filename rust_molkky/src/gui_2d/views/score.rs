use crate::core::{Id, ScoreBoard, ScoreState, Team};
use dioxus::prelude::*;

#[component]
pub fn Scores(teams: ReadSignal<Vec<Team>>) -> Element {
    rsx! {
        table {
            caption { "得点表" }
            thead {
                tr {
                    th { scope: "col", "チーム" }
                    th { scope: "col", "合計" }
                    th { scope: "col", "投擲履歴" }
                    th { scope: "col", "状態" }
                }
            }
            tbody {
                for team in teams.read().iter() {
                    TeamScore {
                        key: "{team.id()}",
                        teams,
                        id: team.id(),
                    }
                }
            }
        }
    }
}

#[component]
fn TeamScore(teams: ReadSignal<Vec<Team>>, id: Id) -> Element {
    let teams = teams.read();
    let Some(team) = teams.iter().find(|team| team.id() == id) else {
        return rsx! {};
    };
    let name = team.name().as_str();
    let board = team.score_board();
    let total = board.total();
    let history = board
        .history()
        .iter()
        .map(|point| point.value().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let state = match board.state() {
        ScoreState::Calculated => "通常".to_owned(),
        ScoreState::Reset => format!("{}点に補正", ScoreBoard::RESET_SCORE),
        ScoreState::Disqualified => "失格".to_owned(),
        ScoreState::Won => "勝利".to_owned(),
    };
    rsx! {
        tr {
            "data-team": "{id}",
            aria_current: if team.turn() { "true" } else { "false" },
            th {
                scope: "row",
                if team.turn() { "▶ " }
                "{name}"
            }
            td { "{total}点" }
            td { "{history}" }
            td { "{state}" }
        }
    }
}
