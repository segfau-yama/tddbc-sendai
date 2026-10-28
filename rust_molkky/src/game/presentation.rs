use super::FieldLayout;
use crate::view::{
    field::FieldTui, molkky::MolkkyTui, score::ScoreBoardTui, skittle::SkittleTui, stick::StickTui,
    team::TeamTui,
};
use rust_molkky::core::{
    CONSECUTIVE_ZERO_LIMIT, Field, GameStatus, Molkky, Point, RESET_SCORE, ScoreBoard,
    TARGET_SCORE, Team,
};
use rust_molkky::simulation::Stick;

pub(crate) fn compose_score(board: &ScoreBoard) -> ScoreBoardTui {
    let mut notice = String::new();
    if board.was_reset() {
        notice.push_str(&format!(
            "  {TARGET_SCORE}点を超過したため{RESET_SCORE}点へ"
        ));
    }
    if board.is_disqualified() {
        notice.push_str(&format!("  {CONSECUTIVE_ZERO_LIMIT}回連続0点のため失格"));
    }
    ScoreBoardTui {
        value: board.value(),
        target: TARGET_SCORE,
        history: board.history().iter().map(Point::value).collect(),
        notice,
    }
}

pub(crate) fn compose_game(
    rules: &dyn Molkky,
    teams: &[Team],
    field: &dyn Field,
    stick: &Stick,
    layout: &FieldLayout,
    active: usize,
    message: &str,
) -> MolkkyTui {
    let status = rules.judge_game(teams);
    let winner = match status {
        GameStatus::InProgress => None,
        GameStatus::Finished { winner } => winner,
    };
    let banner = match status {
        GameStatus::Finished {
            winner: Some(team_index),
        } => format!(
            " MÖLKKY  |  {}の勝利！  Rで新しいゲーム",
            teams[team_index].name()
        ),
        GameStatus::Finished { winner: None } => {
            " MÖLKKY  |  全チームが失格。勝者なし。Rで新しいゲーム".into()
        }
        GameStatus::InProgress => format!(
            " MÖLKKY  |  {}の投てき  |  {}チーム",
            teams[active].name(),
            teams.len()
        ),
    };
    let teams = teams
        .iter()
        .enumerate()
        .map(|(index, team)| TeamTui {
            name: team.name().as_str().to_owned(),
            board: compose_score(team.score_board()),
            active: index == active,
            winner: winner == Some(index),
            disqualified: team.score_board().is_disqualified(),
        })
        .collect();
    let field = FieldTui {
        pins: field
            .skittles()
            .iter()
            .map(|pin| SkittleTui {
                number: pin.number(),
                position: layout.positions[&pin.id()],
                probability: stick.fall_probability(),
            })
            .collect(),
        stick: StickTui {
            position: layout.stick,
        },
    };
    MolkkyTui {
        teams,
        field,
        active,
        banner,
        message: message.into(),
        help: format!(
            "表示位置: {}   {TARGET_SCORE}点ぴったりで勝利 / 超過すると{RESET_SCORE}点",
            layout.stick.label()
        ),
    }
}

pub(crate) fn throw_message(team: &Team, turn: usize, before: i32, point: Point) -> String {
    format!(
        "第{turn}投  {}: {}点  得点 {before} → {}{}",
        team.name(),
        point.value(),
        team.score_board().value(),
        compose_score(team.score_board()).notice
    )
}

#[cfg(test)]
#[path = "../../tests/game/presentation.rs"]
mod tests;
