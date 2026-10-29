use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};

/// クロージャで乱数源や物理世界を捕捉できる、判定関数の可変借用。
pub type JudgeStrategy<'judge> = &'judge mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>;

/// 投てき判定・採点・勝者と次のチームの選択をまとめるドメインサービス。
/// 履歴や一覧を所有せず、入力を変更しない。
pub trait Molkky {
    fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
        judge(stick, skittles)
    }

    /// この一覧を採点する。今回倒れた個体の選別は呼び出し側で行う。
    fn calc_score(&self, fallen_skittles: &[Skittle]) -> Point {
        Point::new(match fallen_skittles {
            [skittle] => skittle.number(),
            skittles => skittles.len() as i32,
        })
    }

    /// 50点到達、または他チームが全て失格したときの勝者を返す。
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

    /// 現在の次から失格者を飛ばし、次に投げるチームを参照で返す。
    fn next_team<'teams>(&self, teams: &'teams [Team]) -> Option<&'teams Team> {
        let remaining = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified)
            .count();
        // 50点到達、または他の全チームの失格でセットは終了する。
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

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
