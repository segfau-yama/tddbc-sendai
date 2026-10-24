use rand::Rng;

use crate::{Skittle, Throw};

/// 各ピンが独立に確率1/2で倒れるモデル。乱数源は呼び出し側で指定する。
pub fn random_throw(rng: &mut impl Rng) -> Throw {
    let skittles: Vec<_> = (1..=12)
        .filter(|_| rng.random_bool(0.5))
        .map(Skittle::new)
        .collect();
    Throw::new(&skittles)
}
