use crate::core::models::{Field, Skittle, validate_unique_ids};
use std::collections::HashSet;

/// 入力を変更せず生成したFieldと、一投で倒れた個体。
pub struct ThrowOutcome<F> {
    field: F,
    knocked_over: Vec<Skittle>,
}
impl<F> ThrowOutcome<F> {
    pub fn new(field: F, knocked_over: Vec<Skittle>) -> Self {
        Self {
            field,
            knocked_over,
        }
    }
    pub fn field(&self) -> &F {
        &self.field
    }
    pub fn knocked_over(&self) -> &[Skittle] {
        &self.knocked_over
    }
    pub fn into_parts(self) -> (F, Vec<Skittle>) {
        (self.field, self.knocked_over)
    }
}

/// 一投全体を計算する。エンジン固有の状態や入力は実装側に持たせる。
pub trait ThrowStrategy<F: Field> {
    type Error;
    fn throw(&mut self, field: &F) -> Result<ThrowOutcome<F>, Self::Error>;
}

/// Strategyの結果が、入力と同じ個体集合を保持することを検証する。
/// エンジンのErrorは伝播し、重複個体・対象外個体などの契約違反はpanicで拒否する。
pub fn throw_stick<F: Field, S: ThrowStrategy<F> + ?Sized>(
    field: &F,
    strategy: &mut S,
) -> Result<ThrowOutcome<F>, S::Error> {
    validate_unique_ids(field.skittles());
    let outcome = strategy.throw(field)?;
    validate_unique_ids(outcome.field.skittles());
    validate_unique_ids(&outcome.knocked_over);
    let targets: HashSet<_> = field.skittles().iter().copied().collect();
    let updated: HashSet<_> = outcome.field.skittles().iter().copied().collect();
    assert_eq!(
        targets, updated,
        "投てき後にスキットルの個体集合が変わっています"
    );
    assert!(
        outcome.knocked_over.iter().all(|pin| targets.contains(pin)),
        "投てき対象外のスキットルが含まれています"
    );
    Ok(outcome)
}
