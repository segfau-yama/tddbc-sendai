use super::{FieldLayout, winning_team};
use crate::view::{
    field::FieldTui, molkky::MolkkyTui, score::ScoreBoardTui, skittle::SkittleTui, stick::StickTui,
    team::TeamTui,
};
use rust_molkky::core::{
    ZERO_POINTS_LIMIT, Point, RESET_SCORE, ScoreBoard, ScoreState, TARGET_SCORE, Team,
};
use rust_molkky::simulation::{ArrayField, RandomStick};

pub(crate) fn compose_score(board: &ScoreBoard) -> ScoreBoardTui {
    let mut notice = String::new();
    if board.state() == ScoreState::Reset {
        notice.push_str(&format!(
            "  {TARGET_SCORE}点を超過したため{RESET_SCORE}点へ"
        ));
    }
    if board.state() == ScoreState::Disqualified {
        notice.push_str(&format!("  {ZERO_POINTS_LIMIT}回連続0点のため失格"));
    }
    ScoreBoardTui {
        value: board.total(),
        target: TARGET_SCORE,
        history: board.history().iter().map(Point::value).collect(),
        notice,
    }
}

pub(crate) fn compose_game(
    teams: &[Team],
    throw_counts: &[usize],
    field: &ArrayField,
    stick: &RandomStick,
    layout: &FieldLayout,
    message: &str,
) -> MolkkyTui {
    let winner = winning_team(teams);
    let current = teams.iter().find(|team| team.turn());
    let active = current
        .and_then(|current| teams.iter().position(|team| team.id() == current.id()))
        .unwrap_or(0);
    let banner = match winner {
        Some(team) => format!(" MÖLKKY  |  {}の勝利！  Rで新しいゲーム", team.name()),
        None if teams
            .iter()
            .all(|team| team.score_board().state() == ScoreState::Disqualified) =>
        {
            " MÖLKKY  |  全チームが失格。勝者なし。Rで新しいゲーム".into()
        }
        None if current.is_some() => format!(
            " MÖLKKY  |  {}の投てき  |  {}チーム",
            teams[active].name(),
            teams.len()
        ),
        None => " MÖLKKY  |  手番が未設定です".into(),
    };
    let teams = teams
        .iter()
        .enumerate()
        .map(|(i, team)| TeamTui {
            name: team.name().as_str().to_owned(),
            board: compose_score(team.score_board()),
            throw_count: throw_counts[i],
            active: team.turn(),
            winner: winner.is_some_and(|winner| winner.id() == team.id()),
            disqualified: team.score_board().state() == ScoreState::Disqualified,
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
        team.score_board().total(),
        compose_score(team.score_board()).notice
    )
}

#[cfg(test)]
#[path = "../../tests/game/presentation.rs"]
mod tests;
