use crate::{Position, Skittle, skittle::validate_unique_numbers};
use rand::Rng;

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
        validate_unique_numbers(&skittles);
        Self { skittles }
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }

    /// 元の配置を保持し、全ピンが隣接マスへ移動した新しい配置を返す。
    #[must_use]
    pub fn relocate(&self, rng: &mut impl Rng) -> Self {
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
        let skittles = self
            .skittles
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
        Self { skittles }
    }
}
