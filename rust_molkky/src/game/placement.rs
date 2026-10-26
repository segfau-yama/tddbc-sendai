use crate::view::field::FieldLayout;
use rand::Rng;
use rust_molkky::core::Skittle;

/// 倒れたピンだけを表示上で移動する。coreのピン集合・確率は変えない。
#[must_use]
pub(crate) fn relocate_skittles(
    layout: &FieldLayout,
    knocked_over: &[Skittle],
    rng: &mut impl Rng,
) -> FieldLayout {
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
    let mut updated = layout.clone();
    for pin in knocked_over {
        let position = layout.positions[&pin.number()];
        let start = rng.random_range(0..DIRECTIONS.len());
        let next = (0..DIRECTIONS.len())
            .find_map(|offset| {
                let (dx, dy) = DIRECTIONS[(start + offset) % DIRECTIONS.len()];
                position.translated(dx, dy)
            })
            .expect("整数座標には範囲内の隣接マスが存在する");
        updated.positions.insert(pin.number(), next);
    }
    updated
}
#[cfg(test)]
#[path = "../../tests/game/placement.rs"]
mod tests;
