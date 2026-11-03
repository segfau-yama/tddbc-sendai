use crate::core::{Field, Skittle};

/// 全チームが共有するフィールドと個体集合。配置は物理側で扱う。
#[derive(Clone)]
pub struct GuiField {
    field: Field,
    skittles: Vec<Skittle>,
}

impl Default for GuiField {
    fn default() -> Self {
        Self::new(Field::new(), (1..=12).map(Skittle::new).collect())
    }
}

impl GuiField {
    pub const BOUNDS: [f32; 4] = [-0.3, -0.15, 0.6, 1.5];
    pub const WALL_THICKNESS: f32 = 0.02;

    pub fn new(field: Field, skittles: Vec<Skittle>) -> Self {
        Self { field, skittles }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }
}
