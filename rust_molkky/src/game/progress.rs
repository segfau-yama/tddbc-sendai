use rust_molkky::{Molkky, ScoreState, Team};

/// アプリの終了条件。coreのWon判定に、既存の最後の生存者の勝利を組み合わせる。
pub(crate) fn winning_team<'a>(rules: &dyn Molkky, teams: &'a [Team]) -> Option<&'a Team> {
    rules.judge_game(teams).or_else(|| {
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
