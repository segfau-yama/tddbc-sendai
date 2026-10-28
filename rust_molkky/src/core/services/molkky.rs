use crate::core::models::{Point, ScoreState, Skittle, Stick, Team};
use std::collections::HashSet;

/// クロージャで乱数源や物理世界を捕捉できる、判定関数の可変借用。
pub type JudgeStrategy<'judge> = &'judge mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>;

/// 個体の検証・投てき判定・採点・勝者と手番の選択をまとめるドメインサービス。
/// 履歴や一覧を所有せず、入力を変更しない。
pub trait Molkky {
    /// 同番号の別個体は許可し、同じIDの二重登録を拒否する。
    fn validate_skittles(&self, skittles: &[Skittle]) {
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
        self.validate_skittles(skittles);
        let before: HashSet<_> = skittles
            .iter()
            .map(|pin| (pin.id(), pin.number()))
            .collect();
        let updated_skittles = judge(stick, skittles);
        self.validate_skittles(&updated_skittles);
        let after: HashSet<_> = updated_skittles
            .iter()
            .map(|pin| (pin.id(), pin.number()))
            .collect();
        assert_eq!(before, after, "判定後のスキットル集合が変わっています");
        updated_skittles
    }

    /// この一覧を採点する。今回倒れた個体の選別は呼び出し側で行う。
    fn calc_score(&self, fallen_skittles: &[Skittle]) -> Point {
        self.validate_skittles(fallen_skittles);
        Point::new(match fallen_skittles {
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
