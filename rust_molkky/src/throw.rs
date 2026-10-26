use rand::Rng;

use crate::{Skittle, Stick, skittle::validate_unique_numbers};

/// 入力の配置を変えず、倒れたスキットルのコピーを返す。
/// 乱数源は更新するため、厳密な意味での純粋関数ではない。
pub fn throw_stick(stick: &Stick, target_skittles: &[Skittle], rng: &mut impl Rng) -> Vec<Skittle> {
    validate_unique_numbers(target_skittles);
    target_skittles
        .iter()
        .copied()
        .filter(|skittle| rng.random_bool(fall_probability(stick, skittle)))
        .collect()
}

/// 投てき条件から倒れる確率を求める。ピンや棒の状態は変更しない。
pub fn fall_probability(stick: &Stick, skittle: &Skittle) -> f64 {
    // 距離0でも定義でき、距離が増えるほど倒れにくくなる既存モデル。
    1.0 / (1.0 + skittle.position().distance_to(stick.position()))
}
