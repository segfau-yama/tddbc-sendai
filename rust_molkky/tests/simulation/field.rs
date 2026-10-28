use rust_molkky::simulation::{ArrayField, StandardFieldGenerator};
use rust_molkky::{Field, FieldGenerator, Skittle};
#[test]
fn 指定したピンと同番号の別個体を保持する() {
    let pins = vec![Skittle::new(12), Skittle::new(12)];
    assert_eq!(ArrayField::new(pins.clone()).skittles(), pins);
}
#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 同じ個体を二重登録できない() {
    ArrayField::new(vec![Skittle::new(5); 2]);
}
#[test]
fn 標準生成は一から十二番を各一本生成する() {
    let field = StandardFieldGenerator.generate();
    assert_eq!(
        field
            .skittles()
            .iter()
            .map(Skittle::number)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
}
#[test]
fn 空のピン集合も表せる() {
    assert!(ArrayField::new(vec![]).skittles().is_empty());
}
