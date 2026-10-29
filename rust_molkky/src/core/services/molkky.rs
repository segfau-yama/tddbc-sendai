use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use itertools::Itertools;

pub trait Molkky {
    fn aim(&self, stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle>;

    fn hit(&self, stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle>;

    fn calc_score(&self, fallen_skittles: &[Skittle]) -> Point {
        Point::new(match fallen_skittles {
            [skittle] => skittle.number(),
            skittles => skittles.len() as i32,
        })
    }

    fn judge_game<'teams>(&self, teams: &'teams [Team]) -> Option<&'teams Team> {
        if let Some(won_team) = teams
            .iter()
            .find(|team| team.score_board().state() == ScoreState::Won)
        {
            return Some(won_team);
        };
        if teams.len() < 2 {
            return None;
        }
        if let Ok(survived_team) = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified)
            .exactly_one()
        {
            return Some(survived_team);
        };
        return None;
    }

    fn next_team<'teams>(&self, teams: &'teams [Team]) -> Option<&'teams Team> {
        let remaining = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified)
            .count();
        if self.judge_game(teams).is_some() || (teams.len() > 1 && remaining == 1) {
            return None;
        }
        let start = teams
            .iter()
            .position(Team::turn)
            .map_or(0, |index| index + 1);
        teams
            .iter()
            .cycle()
            .skip(start)
            .take(teams.len())
            .find(|team| team.score_board().state() != ScoreState::Disqualified)
    }
}
