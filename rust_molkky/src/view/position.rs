/// 描画用の座標。投てきや採点には使わない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Position {
    x: i32,
    y: i32,
}
impl Position {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub(crate) fn x(self) -> i32 {
        self.x
    }
    pub(crate) fn y(self) -> i32 {
        self.y
    }
    pub(crate) fn canvas(self) -> (f64, f64) {
        (f64::from(self.x), f64::from(self.y))
    }
    pub(crate) fn label(self) -> String {
        format!("({}, {})", self.x, self.y)
    }
    pub(crate) fn table_label(self) -> String {
        format!("({:>3},{:>3})", self.x, self.y)
    }
    pub(crate) fn translated(self, dx: i32, dy: i32) -> Option<Self> {
        Some(Self::new(self.x.checked_add(dx)?, self.y.checked_add(dy)?))
    }
}
#[cfg(test)]
#[path = "../../tests/view/position.rs"]
mod tests;
