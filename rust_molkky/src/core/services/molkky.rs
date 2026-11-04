use crate::core::models::{Point, ScoreState, Skittle, SkittleState, Stick, Team};
use itertools::Itertools;

pub trait Molkky {
    type Stick: AsRef<Stick>;
    type Skittle: AsRef<Skittle> + From<Skittle>;

    /// 1刻み進める。投擲条件が無効ならfalse。即時判定では更新不要。
    fn step(&mut self, _stick: &Self::Stick) -> bool {
        true
    }

    /// 即時判定は待機不要。物理計算を使う実装は停止を確認する。
    fn is_settled(&self) -> bool {
        true
    }

    /// 次投へ備える。即時判定は投擲中の状態を持たない。
    fn reset_throw(&mut self) {}

    fn aim(&self, stick: &Self::Stick, skittles: &[Self::Skittle]) -> Vec<Self::Skittle>;

    fn hit(&self, stick: &Self::Stick, skittle: &Self::Skittle) -> Self::Skittle;

    fn chain_hit(
        &self,
        fallen_skittle: &Self::Skittle,
        chained_skittle: &Self::Skittle,
    ) -> Self::Skittle;

    fn stand_up(&self, skittle: &Self::Skittle) -> Self::Skittle {
        skittle.as_ref().with_state(SkittleState::Stand).into()
    }

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
