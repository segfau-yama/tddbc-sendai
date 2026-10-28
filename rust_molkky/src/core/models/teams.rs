use super::{ScoreState, Team};
use std::collections::HashSet;

/// 順序を持つ参加者一覧。元のチームを変更せず、勝者と次の手番を求める。
pub struct Teams<'a> {
    teams: &'a [Team],
}

impl<'a> Teams<'a> {
    pub fn new(teams: &'a [Team]) -> Self {
        Self { teams }
    }

    /// Wonのチームを返す。最後の生存者による勝利はここでは判定しない。
    pub fn judge_game(&self) -> Option<&'a Team> {
        let teams = self.teams;
        assert!(!teams.is_empty(), "チームは1件以上必要です");
        let mut ids = HashSet::new();
        for team in teams {
            assert!(ids.insert(team.id()), "チームIDが重複しています");
        }
        let mut winners = teams
            .iter()
            .filter(|team| team.score_board().state() == ScoreState::Won);
        let winner = winners.next();
        assert!(winners.next().is_none(), "勝者が複数います");
        winner
    }

    /// 次の有効チームへ手番を進めた新しい一覧を返す。終了時は全手番を解除する。
    pub fn next_team(&self) -> Vec<Team> {
        let teams = self.teams;
        let mut turns = teams.iter().enumerate().filter(|(_, team)| team.turn());
        let current = turns.next().map(|(index, _)| index);
        assert!(turns.next().is_none(), "手番のチームが複数います");
        let remaining = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified)
            .count();
        let next = if teams.is_empty()
            || self.judge_game().is_some()
            || (teams.len() >= 2 && remaining == 1)
        {
            None
        } else {
            let start = current.map_or(0, |index| (index + 1) % teams.len());
            (0..teams.len())
                .map(|offset| (start + offset) % teams.len())
                .find(|&index| teams[index].score_board().state() != ScoreState::Disqualified)
        };
        teams
            .iter()
            .enumerate()
            .map(|(index, team)| team.with_turn(next == Some(index)))
            .collect()
    }
}
