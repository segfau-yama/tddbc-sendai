use crate::core::models::{Field, Skittle};
use crate::core::services::{Molkky, SimpleMolkky};

/// フィールドの識別子と、座標を持たない個体集合を合成する。
#[derive(Clone)]
pub struct ArrayField {
    field: Field,
    skittles: Vec<Skittle>,
}
impl ArrayField {
    pub fn new(field: Field, skittles: Vec<Skittle>) -> Self {
        SimpleMolkky.validate_skittles(&skittles);
        Self { field, skittles }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }
}

/// 標準ルールの1〜12各1本を生成する。
pub struct StandardFieldGenerator;
impl StandardFieldGenerator {
    pub fn generate(&self) -> ArrayField {
        ArrayField::new(Field::new(), (1..=12).map(Skittle::new).collect())
    }
}
