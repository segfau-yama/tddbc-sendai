use crate::core::models::{GameStatus, Point, Team};
use std::collections::HashSet;

/// 全参加チームの有効な履歴から、入力を変えずに試合状態を返す。
pub fn judge_game(teams: &[Team]) -> GameStatus {
    assert!(!teams.is_empty(), "チームは1件以上必要です");
    let mut ids = HashSet::new();
    for team in teams {
        assert!(ids.insert(team.id()), "チームIDが重複しています");
        team.score_board().validate_for_game();
    }
    let mut winners = teams
        .iter()
        .enumerate()
        .filter(|(_, team)| team.score_board().is_winner());
    if let Some((team_index, _)) = winners.next() {
        assert!(winners.next().is_none(), "勝者が複数います");
        return GameStatus::Finished {
            winner: Some(team_index),
        };
    }
    let mut remaining = teams
        .iter()
        .enumerate()
        .filter(|(_, team)| !team.score_board().is_disqualified());
    let Some((team_index, _)) = remaining.next() else {
        return GameStatus::Finished { winner: None };
    };
    // 単独参加は、参加しただけでは勝利しない。
    if teams.len() >= 2 && remaining.next().is_none() {
        GameStatus::Finished {
            winner: Some(team_index),
        }
    } else {
        GameStatus::InProgress
    }
}

/// 終了後はNone、それ以外は失格チームを飛ばした次のインデックスを返す。
pub fn next_team(teams: &[Team], current: usize) -> Option<usize> {
    let status = judge_game(teams);
    assert!(current < teams.len(), "チームが存在しません");
    if matches!(status, GameStatus::Finished { .. }) {
        return None;
    }
    (1..=teams.len())
        .map(|offset| (current + offset) % teams.len())
        .find(|&index| !teams[index].score_board().is_disqualified())
}

/// 試合の記録制約を検証し、元のチーム一覧を変更せずに点数を追加する。
#[must_use]
pub fn record_point(teams: &[Team], team_index: usize, point: Point) -> Vec<Team> {
    assert!(
        matches!(judge_game(teams), GameStatus::InProgress),
        "ゲームは終了しています"
    );
    assert!(team_index < teams.len(), "チームが存在しません");
    assert!(
        !teams[team_index].score_board().is_disqualified(),
        "チームは失格しています"
    );
    let mut updated = teams.to_vec();
    updated[team_index] =
        teams[team_index].with_score_board(teams[team_index].score_board().add(point));
    judge_game(&updated);
    updated
}
