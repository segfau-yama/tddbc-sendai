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
