use rust_molkky::simulation::{ArrayField, StandardFieldGenerator};
use rust_molkky::{Field, Skittle};
#[test]
fn 指定したピンと同番号の別個体を保持する() {
    let pins = vec![Skittle::new(12), Skittle::new(12)];
    let identity = Field::new();
    let field = ArrayField::new(identity, pins.clone());
    assert_eq!(field.skittles(), pins);
    assert_eq!(field.field().id(), identity.id());
    assert_eq!(field.clone().field().id(), identity.id());
}
#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 同じ個体を二重登録できない() {
    ArrayField::new(Field::new(), vec![Skittle::new(5); 2]);
}
#[test]
fn 標準生成は一から十二番を各一本生成する() {
    let field = StandardFieldGenerator.generate();
    assert_ne!(
        field.field().id(),
        StandardFieldGenerator.generate().field().id()
    );
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
    assert!(ArrayField::new(Field::new(), vec![]).skittles().is_empty());
}
