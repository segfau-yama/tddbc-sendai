use crate::core::models::{Skittle, Stick, validate_unique_numbers};
use rand::Rng;

/// 入力を変えず、棒の設定に基づく独立抽選で倒れたピンを返す。
/// 乱数源は更新するため、厳密な意味での純粋関数ではない。
pub fn throw_stick(stick: &Stick, target_skittles: &[Skittle], rng: &mut impl Rng) -> Vec<Skittle> {
    validate_unique_numbers(target_skittles);
    target_skittles
        .iter()
        .copied()
        .filter(|_| rng.random_bool(stick.fall_probability()))
        .collect()
}
