use rust_molkky::core::Point;

pub(super) struct PointTui<'a> {
    point: &'a Point,
}

impl<'a> PointTui<'a> {
    pub(super) fn new(point: &'a Point) -> Self {
        Self { point }
    }

    pub(super) fn label(&self) -> String {
        self.point.value().to_string()
    }
}
