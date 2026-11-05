use crate::core::Field;

/// 全チームが共有するフィールドの識別子と範囲。
#[derive(Clone, PartialEq)]
pub struct GuiField {
    field: Field,
}

impl GuiField {
    pub const BOUNDS: [f32; 4] = [-0.3, -0.15, 0.6, 1.5];
    pub const WALL_THICKNESS: f32 = 0.02;
    pub const THROW_POSITION: [f32; 2] = [0.0, 0.0];

    pub fn new(field: Field) -> Self {
        Self { field }
    }
}

impl AsRef<Field> for GuiField {
    fn as_ref(&self) -> &Field {
        &self.field
    }
}
