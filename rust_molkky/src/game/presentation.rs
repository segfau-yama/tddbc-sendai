use crate::view::{
    field::{FieldLayout, FieldTui},
    molkky::MolkkyTui,
    score::ScoreBoardTui,
    skittle::SkittleTui,
    stick::StickTui,
    team::TeamTui,
};
use rust_molkky::core::{
    CONSECUTIVE_ZERO_LIMIT, GameStatus, Molkky, Point, RESET_SCORE, ScoreBoard, TARGET_SCORE, Team,
};

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
    game: &Molkky,
    layout: &FieldLayout,
    active: usize,
    message: &str,
) -> MolkkyTui {
    let winner = game.result();
    let banner = match game.status() {
        GameStatus::Won { team_index } => format!(
            " MÖLKKY  |  {}の勝利！  Rで新しいゲーム",
            game.teams()[team_index].name()
        ),
        GameStatus::FinishedWithoutWinner => {
            " MÖLKKY  |  全チームが失格。勝者なし。Rで新しいゲーム".into()
        }
        GameStatus::InProgress => format!(
            " MÖLKKY  |  {}の投てき  |  {}チーム",
            game.teams()[active].name(),
            game.teams().len()
        ),
    };
    let teams = game
        .teams()
        .iter()
        .enumerate()
        .map(|(index, team)| TeamTui {
            name: team.name().to_owned(),
            board: compose_score(team.score_board()),
            active: index == active,
            winner: winner == Some(index),
            disqualified: team.score_board().is_disqualified(),
        })
        .collect();
    let field = FieldTui {
        pins: game
            .field()
            .skittles()
            .iter()
            .map(|pin| SkittleTui {
                number: pin.number(),
                position: layout.positions[&pin.number()],
                probability: game.stick().fall_probability(),
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
