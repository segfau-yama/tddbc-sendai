pub(crate) struct PointTui {
    point: i32,
}
impl PointTui {
    pub(crate) fn new(point: i32) -> Self {
        Self { point }
    }
    pub(crate) fn label(&self) -> String {
        self.point.to_string()
    }
}
