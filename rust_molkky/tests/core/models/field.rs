use rust_molkky::{Field, Skittle};
#[test]
fn 座標なしで指定したピン集合を保持する() {
    let pins = vec![Skittle::new(5), Skittle::new(8)];
    assert_eq!(Field::new(pins.clone()).skittles(), pins);
}
#[test]
#[should_panic(expected = "スキットル番号が重複しています: 5")]
fn 同じ番号のピンを二本持てない() {
    Field::new(vec![Skittle::new(5); 2]);
}
#[test]
fn 既定のピン集合は一から十二番を一つずつ持つ() {
    assert_eq!(
        Field::default()
            .skittles()
            .iter()
            .map(Skittle::number)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
}
#[test]
fn 空のピン集合も表せる() {
    assert!(Field::new(vec![]).skittles().is_empty());
}
