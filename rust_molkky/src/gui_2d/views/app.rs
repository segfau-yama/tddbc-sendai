use super::{Field, Result, Scores};
use crate::core::{Molkky, SkittleState, StandUp, StickState, Team};
use crate::gui_2d::{
    models::{GuiSkittle, GuiStick},
    services::{
        GuiMolkky,
        field::{create_field, create_world},
        teams::from_names,
    },
};
use dioxus::prelude::*;

pub(super) fn advance_turn(teams: &mut [Team], rules: &GuiMolkky) {
    let winner = rules.judge_game(teams).map(Team::id);
    let next = rules.next_team(teams).map(Team::id);
    for team in teams {
        *team = team.with_turn(next == Some(team.id()));
        if winner == Some(team.id()) {
            *team = team.with_score_board(team.score_board().with_win());
        }
    }
}

pub(super) fn finish_throw(
    teams: &mut [Team],
    stick: &mut GuiStick,
    skittles: &mut [GuiSkittle],
    rules: &mut GuiMolkky,
) {
    if stick.as_ref().state() != StickState::Stop
        || skittles
            .iter()
            .any(|s| s.as_ref().state() == SkittleState::Hit)
    {
        return;
    }
    let Some(current) = teams.iter().position(Team::turn) else {
        return;
    };
    let fallen: Vec<_> = skittles
        .iter()
        .filter(|s| s.as_ref().state() == SkittleState::Fall)
        .map(|s| *s.as_ref())
        .collect();
    let team = &teams[current];
    teams[current] = team.with_score_board(team.score_board().add(rules.calc_score(&fallen)));
    advance_turn(teams, rules);
    // 同じStopを二度採点しない。次投も同じ個体を保持状態から始める。
    *stick = GuiStick::new(*stick.as_ref(), None, 0.0);
    if teams.iter().any(Team::turn) {
        for skittle in skittles.iter_mut() {
            *skittle = skittle.stand_up();
        }
        // 立て直した配置から、前投の速度と接触履歴を持たないワールドを用意する。
        *rules = create_world(stick, skittles);
    }
}

#[component]
pub fn App() -> Element {
    let (field, mut stick, mut skittles, mut rules) = use_hook(|| {
        let (field, stick, skittles) = create_field();
        let rules = create_world(&stick, &skittles);
        (
            field,
            Signal::new(stick),
            Signal::new(skittles),
            Signal::new(rules),
        )
    });
    let mut teams = use_signal(|| {
        let mut teams = from_names(&["赤チーム", "青チーム"]);
        advance_turn(&mut teams, &rules.peek());
        teams
    });
    // 参照を作り直すと、propsの更新が共有元の購読を繰り返し無効化する。
    let score_teams = use_hook(|| ReadSignal::from(teams));
    let enabled = use_memo(move || teams.read().iter().any(Team::turn));
    let complete = move |_| {
        finish_throw(
            &mut teams.write(),
            &mut stick.write(),
            &mut skittles.write(),
            &mut rules.write(),
        );
    };
    rsx! {
        main {
            document::Title { "モルック" }
            style { {include_str!("style.css")} }
            h1 { "モルック" }
            div {
                class: "game-layout",
                Field {
                    field,
                    stick,
                    skittles,
                    rules,
                    enabled,
                    on_complete: complete,
                }
                div {
                    class: "scores",
                    Scores { teams: score_teams }
                    if !enabled() {
                        Result {
                            winner: rules.read().judge_game(&teams.read()).map(|team| team.name().to_string()),
                        }
                    }
                }
            }
        }
    }
}
