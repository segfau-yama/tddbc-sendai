use super::Skittle;

/// 配列・座標・物理世界など、Fieldの表現に依存しない採点対象。
pub trait Field {
    fn skittles(&self) -> &[Skittle];
}

/// ルールや用途に応じた具体的なFieldを生成するStrategy。
pub trait FieldGenerator {
    type Field: Field;
    fn generate(&self) -> Self::Field;
}
