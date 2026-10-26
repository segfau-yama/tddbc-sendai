use rand::Rng;

use crate::{Skittle, Stick, skittle::validate_unique_numbers};

/// 入力の配置を変えず、倒れたスキットルのコピーを返す。
/// 乱数源は更新するため、厳密な意味での純粋関数ではない。
pub fn throw_stick(stick: &Stick, target_skittles: &[Skittle], rng: &mut impl Rng) -> Vec<Skittle> {
    validate_unique_numbers(target_skittles);
    target_skittles
        .iter()
        .copied()
        .filter(|skittle| rng.random_bool(skittle.fall_probability(stick.position())))
        .collect()
}
