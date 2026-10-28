use rust_molkky::{ScoreState, Team, Teams};

/// アプリの終了条件。coreのWon判定に、既存の最後の生存者の勝利を組み合わせる。
pub(crate) fn winning_team(teams: &[Team]) -> Option<&Team> {
    Teams::new(teams).judge_game().or_else(|| {
        if teams.len() < 2 {
            return None;
        }
        let mut remaining = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified);
        let team = remaining.next()?;
        remaining.next().is_none().then_some(team)
    })
}

#[cfg(test)]
#[path = "../../tests/game/progress.rs"]
mod tests;
