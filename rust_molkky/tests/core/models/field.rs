use rust_molkky::{Field, FieldGenerator, Skittle};

#[derive(Clone)]
struct CoordinateField {
    pins: Vec<Skittle>,
    positions: Vec<(f64, f64)>,
}
impl Field for CoordinateField {
    fn skittles(&self) -> &[Skittle] {
        &self.pins
    }
}
struct DuplicateTwelves;
impl FieldGenerator for DuplicateTwelves {
    type Field = CoordinateField;
    fn generate(&self) -> CoordinateField {
        CoordinateField {
            pins: vec![Skittle::new(12), Skittle::new(12)],
            positions: vec![(0.0, 0.0), (1.0, 0.0)],
        }
    }
}

#[test]
fn 生成方法で座標付きの独自fieldを返し同番号の別個体を配置できる() {
    let generator: &dyn FieldGenerator<Field = CoordinateField> = &DuplicateTwelves;
    let field = generator.generate();
    assert_eq!(field.skittles().len(), 2);
    assert_ne!(field.skittles()[0].id(), field.skittles()[1].id());
    assert_eq!(field.positions, [(0.0, 0.0), (1.0, 0.0)]);
    let erased: &dyn Field = &field;
    assert_eq!(erased.skittles(), field.skittles());
}
