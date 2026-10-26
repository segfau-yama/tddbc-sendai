use rust_molkky::core::Position;

pub(super) struct PositionTui<'a> {
    position: &'a Position,
}

impl<'a> PositionTui<'a> {
    pub(super) fn new(position: &'a Position) -> Self {
        Self { position }
    }

    pub(super) fn canvas(&self) -> (f64, f64) {
        (f64::from(self.position.x()), f64::from(self.position.y()))
    }

    pub(super) fn label(&self) -> String {
        format!("({}, {})", self.position.x(), self.position.y())
    }

    pub(super) fn table_label(&self) -> String {
        format!("({:>3},{:>3})", self.position.x(), self.position.y())
    }
}
