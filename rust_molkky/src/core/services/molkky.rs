use crate::core::models::{Point, ScoreState, Skittle, Stick, Team, validate_unique_ids};
use rand::{Rng, RngCore};
use std::collections::HashSet;

/// 投てき・採点・試合進行のルール。既定メソッドは現在のシンプルなモルックのルールを使う。
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

    /// 勝者のインデックスを返す。継続可否はnext_teamで判定する。
    fn judge_game(&self, teams: &[Team]) -> Option<usize> {
        assert!(!teams.is_empty(), "チームは1件以上必要です");
        let mut ids = HashSet::new();
        for team in teams {
            assert!(ids.insert(team.id()), "チームIDが重複しています");
        }
        let mut winners = teams
            .iter()
            .enumerate()
            .filter(|(_, team)| team.score_board().state() == ScoreState::Won);
        if let Some((team_index, _)) = winners.next() {
            assert!(winners.next().is_none(), "勝者が複数います");
            return Some(team_index);
        }
        let mut remaining = teams
            .iter()
            .enumerate()
            .filter(|(_, team)| team.score_board().state() != ScoreState::Disqualified);
        let (team_index, _) = remaining.next()?;
        // 単独参加は、参加しただけでは勝利しない。
        if teams.len() >= 2 && remaining.next().is_none() {
            Some(team_index)
        } else {
            None
        }
    }

    /// 終了後はNone、それ以外は失格チームを飛ばした次のインデックスを返す。
    fn next_team(&self, teams: &[Team], current: usize) -> Option<usize> {
        let winner = self.judge_game(teams);
        assert!(current < teams.len(), "チームが存在しません");
        if winner.is_some() {
            return None;
        }
        (1..=teams.len())
            .map(|offset| (current + offset) % teams.len())
            .find(|&index| teams[index].score_board().state() != ScoreState::Disqualified)
    }

    /// 試合の記録制約を検証し、元のチーム一覧を変更せずに点数を追加する。
    fn record_point(&self, teams: &[Team], team_index: usize, point: Point) -> Vec<Team> {
        assert!(
            self.next_team(teams, team_index).is_some(),
            "ゲームは終了しています"
        );
        let team = teams.get(team_index).expect("チームが存在しません");
        assert!(
            team.score_board().state() != ScoreState::Disqualified,
            "チームは失格しています"
        );
        let mut updated = teams.to_vec();
        updated[team_index] = team.with_score_board(team.score_board().add(point));
        self.judge_game(&updated);
        updated
    }
}

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
