use rand::Rng;

use crate::{Field, Position, Throw};

/// 配置されたピンから投てき結果を生成する。
pub fn random_throw(field: &mut Field, from: Position, rng: &mut impl Rng) -> Throw {
    let skittles: Vec<_> = field
        .skittles()
        .iter()
        .copied()
        .filter(|skittle| rng.random_bool(skittle.fall_probability(from)))
        .collect();
    let thrown = Throw::new(&skittles);
    // 確定した投てき結果には移動前の座標を保持し、判定後に全ピンを移す。
    field.relocate(rng);
    thrown
}
