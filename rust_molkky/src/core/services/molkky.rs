use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use std::collections::HashSet;

/// クロージャで乱数源や物理世界を捕捉できる、判定関数の借用。ヒープ確保は不要。
pub type JudgeStrategy<'a> = &'a mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>;

/// 衝突・転倒判定・採点・チーム選択のルール。既定メソッドは現在のシンプルなモルックのルールを使う。
/// 履歴やFieldを所有せず、手番は新しいチーム一覧で返し、得点の更新は呼び出し側が扱う。
pub trait Molkky {
    /// 同番号の別個体は許可し、同じIDの二重登録を拒否する。
    fn validate_unique_ids(&self, skittles: &[Skittle]) {
        let mut ids = HashSet::new();
        for skittle in skittles {
            assert!(
                ids.insert(skittle.id()),
                "スキットルIDが重複しています: {}",
                skittle.id()
            );
        }
    }

    /// 判定方法を注入し、同じ個体集合の新しい状態を返す。入力は変更しない。
    fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
        self.validate_unique_ids(skittles);
        let updated = judge(stick, skittles);
        self.validate_unique_ids(&updated);
        let before: HashSet<_> = skittles
            .iter()
            .map(|pin| (pin.id(), pin.number()))
            .collect();
        let after: HashSet<_> = updated.iter().map(|pin| (pin.id(), pin.number())).collect();
        assert_eq!(before, after, "判定後のスキットル集合が変わっています");
        updated
    }

    /// 位置や投てき方法に依存せず、倒れた番号だけから採点する。
    fn calc_score(&self, knocked_over_skittles: &[Skittle]) -> Point {
        self.validate_unique_ids(knocked_over_skittles);
        Point::new(match knocked_over_skittles {
            [skittle] => skittle.number(),
            skittles => i32::try_from(skittles.len()).expect("スキットル本数が得点の範囲外です"),
        })
    }

    /// Wonのチームを返す。最後の生存者による勝利はここでは判定しない。
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

    /// 次の有効チームへ手番を進めた新しい一覧を返す。終了時は全手番を解除する。
    fn next_team(&self, teams: &[Team]) -> Vec<Team> {
        let mut turns = teams.iter().enumerate().filter(|(_, team)| team.turn());
        let current = turns.next().map(|(index, _)| index);
        assert!(turns.next().is_none(), "手番のチームが複数います");
        let remaining = teams
            .iter()
            .filter(|team| team.score_board().state() != ScoreState::Disqualified)
            .count();
        let next = if teams.is_empty()
            || self.judge_game(teams).is_some()
            || (teams.len() >= 2 && remaining == 1)
        {
            None
        } else {
            let start = current.map_or(0, |index| (index + 1) % teams.len());
            (0..teams.len())
                .map(|offset| (start + offset) % teams.len())
                .find(|&index| teams[index].score_board().state() != ScoreState::Disqualified)
        };
        teams
            .iter()
            .enumerate()
            .map(|(index, team)| team.with_turn(next == Some(index)))
            .collect()
    }
}

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
