use crate::core::models::{GameStatus, Team};

/// 全参加チームの有効な履歴から、入力を変えずに試合状態を返す。
pub fn judge_game(teams: &[Team]) -> GameStatus {
    assert!(!teams.is_empty(), "チームは1件以上必要です");
    for team in teams {
        team.score_board().validate_for_game();
    }
    let mut winners = teams
        .iter()
        .enumerate()
        .filter(|(_, team)| team.score_board().is_winner());
    if let Some((team_index, _)) = winners.next() {
        assert!(winners.next().is_none(), "勝者が複数います");
        return GameStatus::Won { team_index };
    }
    let mut remaining = teams
        .iter()
        .enumerate()
        .filter(|(_, team)| !team.score_board().is_disqualified());
    let Some((team_index, _)) = remaining.next() else {
        return GameStatus::FinishedWithoutWinner;
    };
    // 単独参加は、参加しただけでは勝利しない。
    if teams.len() >= 2 && remaining.next().is_none() {
        GameStatus::Won { team_index }
    } else {
        GameStatus::InProgress
    }
}
