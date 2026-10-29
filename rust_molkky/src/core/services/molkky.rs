use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};

pub type JudgeStrategy<'judge> = &'judge mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>;

pub trait Molkky {
    fn aim(&self, stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle>;

    fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
        judge(stick, skittles)
    }

    fn calc_score(&self, fallen_skittles: &[Skittle]) -> Point {
        Point::new(match fallen_skittles {
            [skittle] => skittle.number(),
            skittles => skittles.len() as i32,
        })
    }

    fn judge_game<'teams>(&self, teams: &'teams [Team]) -> Option<&'teams Team> {
        if let Some(team) = teams
            .iter()
            .find(|team| team.score_board().state() == ScoreState::Won)
        {
            return Some(team);
        }
        let mut survived_teams = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified);
        match (survived_teams.next(), survived_teams.next()) {
            (Some(team), None) if teams.len() > 1 => Some(team),
            _ => None,
        }
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

#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
