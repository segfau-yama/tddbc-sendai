use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use std::collections::HashSet;
use itertools::Itertools;

/// クロージャで乱数源や物理世界を捕捉できる、判定関数の可変借用。
pub type JudgeStrategy<'judge> = &'judge mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>;

/// 個体の検証・投てき判定・採点・勝者と手番の選択をまとめるドメインサービス。
/// 履歴や一覧を所有せず、入力を変更しない。
pub trait Molkky {
    fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
        let updated_skittles = judge(stick, skittles);
        if updated_skittles.iter().counts().any(|(k, v)| v > 1) {
            return updated_skittles;
        }
        return updated_skittles;
    }

    /// この一覧を採点する。今回倒れた個体の選別は呼び出し側で行う。
    fn calc_score(&self, fallen_skittles: &[Skittle]) -> Option<Point> {
        if fallen_skittles.iter().counts().any(|(k, v)| v > 1) {
            return Some(Point::new(match fallen_skittles {
                [skittle] => skittle.number(),
                skittles => skittles.len() as i32,
            }));
        }
        return None;
    }

    /// Wonのチームを返す。最後の生存者による勝利はここでは判定しない。
    fn judge_game(&self, teams: &[Team]) -> Option<&Team> {
        let mut winners = teams
            .iter()
            .filter(|team| team.score_board().state() == ScoreState::Won);
        let winner = winners.next();
        winner
    }

    /// 次の有効チームへ手番を進めた新しい一覧を返す。終了時は全手番を解除する。
    fn next_team(&self, teams: &[Team]) -> Option<&Team> {
        let turns = teams.iter().filter(|team| team.turn());
        let current = turns.next();
        current
    }
}

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
