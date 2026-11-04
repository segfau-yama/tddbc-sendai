use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use itertools::Itertools;

pub trait Molkky {
    type Stick: AsRef<Stick>;
    type Skittle: AsRef<Skittle>;
    /// 投擲方式ごとの結果。確率や初速などを同じ数値型に固定しない。
    type ThrowOutput;

    fn aim(&self, stick: &Self::Stick, skittles: &[Self::Skittle]) -> Vec<Skittle>;

    fn throw(&mut self, stick: &Self::Stick) -> Self::ThrowOutput;

    /// 更新後の個体と、今回StandからFallへ変わったかを返す。
    fn hit(&self, stick: &Self::Stick, skittle: &Self::Skittle) -> (Skittle, bool);

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
        None
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
