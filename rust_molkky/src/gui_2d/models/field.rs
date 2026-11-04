use super::GuiSkittle;
use crate::core::Field;

/// 全チームが共有するフィールドと個体集合。現在位置は物理側で扱う。
#[derive(Clone)]
pub struct GuiField {
    field: Field,
    skittles: Vec<GuiSkittle>,
}

impl GuiField {
    pub const BOUNDS: [f32; 4] = [-0.3, -0.15, 0.6, 1.5];
    pub const WALL_THICKNESS: f32 = 0.02;
    pub const THROW_POSITION: [f32; 2] = [0.0, 0.0];

    pub fn new(field: Field, skittles: Vec<GuiSkittle>) -> Self {
        Self { field, skittles }
    }

    pub fn skittles(&self) -> &[GuiSkittle] {
        &self.skittles
    }
}

impl AsRef<Field> for GuiField {
    fn as_ref(&self) -> &Field {
        &self.field
    }
}
