use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use rand::{Rng, RngCore};
use std::collections::HashSet;

pub(crate) fn validate_unique_ids(skittles: &[Skittle]) {
    let mut ids = HashSet::new();
    for skittle in skittles {
        assert!(
            ids.insert(skittle.id()),
            "スキットルIDが重複しています: {}",
            skittle.id()
        );
    }
}

/// 投てき・採点・チーム選択のルール。既定メソッドは現在のシンプルなモルックのルールを使う。
/// 履歴やFieldを所有せず、更新は新しいチーム一覧として返す。
pub trait Molkky {
    /// 各個体を棒の設定確率で独立抽選する。入力は変更せず、乱数源だけを更新する。
    fn throw(&self, stick: &Stick, skittles: &[Skittle], rng: &mut dyn RngCore) -> Vec<Skittle> {
        validate_unique_ids(skittles);
        skittles
            .iter()
            .copied()
            .filter(|_| rng.random_bool(stick.fall_probability()))
            .collect()
    }

    /// 位置や投てき方法に依存せず、倒れた番号だけから採点する。
    fn calc_score(&self, knocked_over_skittles: &[Skittle]) -> Point {
        validate_unique_ids(knocked_over_skittles);
        Point::new(match knocked_over_skittles {
            [skittle] => skittle.number(),
            skittles => i32::try_from(skittles.len()).expect("スキットル本数が得点の範囲外です"),
        })
    }

    /// Wonのチームを返す。手番・失格スキップ・試合進行は呼び出し側が扱う。
    fn judge_game<'a>(&self, teams: &'a [Team]) -> Option<&'a Team> {
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

    /// turn=trueのチームを返す。手番フラグは変更しない。
    fn next_team<'a>(&self, teams: &'a [Team]) -> Option<&'a Team> {
        let mut turns = teams.iter().filter(|team| team.turn());
        let team = turns.next();
        assert!(turns.next().is_none(), "手番のチームが複数います");
        team
    }
}

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
