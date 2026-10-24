use rand::Rng;
use std::collections::HashSet;

use crate::{Position, Skittle};

pub struct Field {
    skittles: Vec<Skittle>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new(
            (1..=12)
                .map(|number| {
                    Skittle::new(number)
                        .with_position(Position::new((number - 1) % 3 - 1, (number - 1) / 3 + 3))
                })
                .collect(),
        )
    }
}

impl Field {
    pub fn new(skittles: Vec<Skittle>) -> Self {
        let mut numbers = HashSet::new();
        for skittle in &skittles {
            // 番号は位置にかかわらずフィールド内で一意になる。
            assert!(
                numbers.insert(skittle.number()),
                "スキットル番号が重複しています: {}",
                skittle.number()
            );
        }
        Self { skittles }
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }

    pub(crate) fn relocate(&mut self, rng: &mut impl Rng) {
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
        for skittle in &mut self.skittles {
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
            *skittle = skittle.with_position(next);
        }
    }
}
