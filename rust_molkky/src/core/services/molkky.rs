use crate::core::models::{Skittle, Skittles, Stick};
use std::collections::HashSet;

/// クロージャで乱数源や物理世界を捕捉できる、判定関数の可変借用。
pub type JudgeStrategy<'judge> = &'judge mut dyn FnMut(&Stick, Skittles<'_>) -> Vec<Skittle>;

/// 衝突・転倒の判定を呼び出す。集合の検証・採点・チーム選択はドメインモデルが扱う。
pub trait Molkky {
    /// 判定方法を注入し、同じ個体集合の新しい状態を返す。入力は変更しない。
    fn hit(&self, stick: &Stick, skittles: Skittles<'_>, judge: JudgeStrategy<'_>) -> Vec<Skittle> {
        let before: HashSet<_> = skittles
            .as_slice()
            .iter()
            .map(|pin| (pin.id(), pin.number()))
            .collect();
        let updated_skittles = judge(stick, skittles);
        let updated = Skittles::new(&updated_skittles);
        let after: HashSet<_> = updated
            .as_slice()
            .iter()
            .map(|pin| (pin.id(), pin.number()))
            .collect();
        assert_eq!(before, after, "判定後のスキットル集合が変わっています");
        updated_skittles
    }
}

/// 状態を持たず、Molkkyのデフォルト実装をそのまま利用する。
#[derive(Default)]
pub struct SimpleMolkky;
impl Molkky for SimpleMolkky {}
