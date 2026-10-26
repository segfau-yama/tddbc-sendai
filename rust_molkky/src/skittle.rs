use std::collections::HashSet;

pub(crate) fn validate_unique_numbers(skittles: &[Skittle]) {
    let mut numbers = HashSet::new();
    for skittle in skittles {
        // ピンは番号で識別する。座標が異なっても同じ番号は重複になる。
        assert!(
            numbers.insert(skittle.number()),
            "スキットル番号が重複しています: {}",
            skittle.number()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn x(&self) -> i32 {
        self.x
    }
    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn distance_to(&self, other: Position) -> f64 {
        // 整数の差を先に取ると、座標の端でオーバーフローする。
        (f64::from(self.x) - f64::from(other.x)).hypot(f64::from(self.y) - f64::from(other.y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Skittle {
    number: i32,
    position: Position,
}

impl Skittle {
    /// 1〜12以外の番号はpanicで拒否する。番号にNoneは指定できない。
    pub fn new(number: i32) -> Self {
        // 存在するスキットルの番号を生成時に保証する。
        assert!(
            (1..=12).contains(&number),
            "スキットル番号は1〜12です: {number}"
        );
        Self {
            number,
            position: Position::new(0, 0),
        }
    }

    pub fn number(&self) -> i32 {
        self.number
    }

    pub fn position(&self) -> Position {
        self.position
    }

    #[must_use]
    pub fn with_position(&self, position: Position) -> Self {
        Self {
            number: self.number,
            position,
        }
    }

    pub fn fall_probability(&self, from: Position) -> f64 {
        // 距離0でも確率が定義でき、距離の増加に応じて単調に下がるモデル。
        1.0 / (1.0 + self.position.distance_to(from))
    }
}
