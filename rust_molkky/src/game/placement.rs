use crate::view::position::Position;
use rand::Rng;
use rust_molkky::core::{Id, Skittle};
use std::collections::HashMap;

/// 個体と描画座標の対応。coreのField表現から独立した表示状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldLayout {
    pub(crate) positions: HashMap<Id, Position>,
    pub(crate) stick: Position,
}
impl FieldLayout {
    pub(crate) fn new(pins: &[Skittle]) -> Self {
        // 標準配置の描画座標。同番号の別個体は同じ初期位置から個別に動かせる。
        let rows = [
            (1, -1, 3),
            (2, 1, 3),
            (3, -2, 4),
            (10, 0, 4),
            (4, 2, 4),
            (5, -3, 5),
            (11, -1, 5),
            (12, 1, 5),
            (6, 3, 5),
            (7, -2, 6),
            (9, 0, 6),
            (8, 2, 6),
        ];
        Self {
            positions: pins
                .iter()
                .map(|pin| {
                    let (_, x, y) = rows.iter().find(|(n, _, _)| *n == pin.number()).unwrap();
                    (pin.id(), Position::new(*x, *y))
                })
                .collect(),
            stick: Position::new(0, 0),
        }
    }
}

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
        let position = layout.positions[&pin.id()];
        let start = rng.random_range(0..DIRECTIONS.len());
        let next = (0..DIRECTIONS.len())
            .find_map(|offset| {
                let (dx, dy) = DIRECTIONS[(start + offset) % DIRECTIONS.len()];
                position.translated(dx, dy)
            })
            .expect("整数座標には範囲内の隣接マスが存在する");
        updated.positions.insert(pin.id(), next);
    }
    updated
}
#[cfg(test)]
#[path = "../../tests/game/placement.rs"]
mod tests;
