use crate::core::models::{Field, FieldGenerator, Skittle, validate_unique_ids};

/// 座標を持たない具体的なField。
#[derive(Clone)]
pub struct ArrayField {
    skittles: Vec<Skittle>,
}
impl ArrayField {
    pub fn new(skittles: Vec<Skittle>) -> Self {
        validate_unique_ids(&skittles);
        Self { skittles }
    }
}
impl Field for ArrayField {
    fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }
}

/// 標準ルールの1〜12各1本を生成する。変則構成の制約とは分離する。
pub struct StandardFieldGenerator;
impl FieldGenerator for StandardFieldGenerator {
    type Field = ArrayField;
    fn generate(&self) -> ArrayField {
        ArrayField::new((1..=12).map(Skittle::new).collect())
    }
}
