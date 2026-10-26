use rand::Rng;

use super::{Field, Position};

/// 元の配置を保持し、全ピンを隣接マスへ移動した新しいFieldを返す。
/// 乱数源は更新する。
#[must_use]
pub fn relocate_skittles(field: &Field, rng: &mut impl Rng) -> Field {
    const DIRECTIONS: [(i32, i32); 8] = [
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];
    let skittles = field
        .skittles()
        .iter()
        .map(|skittle| {
            let position = skittle.position();
            let start = rng.random_range(0..DIRECTIONS.len());
            // 座標の端では範囲内の隣接マスを選ぶ。元の位置には留まらない。
            let next = (0..DIRECTIONS.len())
                .find_map(|offset| {
                    let (dx, dy) = DIRECTIONS[(start + offset) % DIRECTIONS.len()];
                    Some(Position::new(
                        position.x().checked_add(dx)?,
                        position.y().checked_add(dy)?,
                    ))
                })
                .expect("整数座標には範囲内の隣接マスが存在する");
            skittle.with_position(next)
        })
        .collect();
    Field::new(skittles)
}
