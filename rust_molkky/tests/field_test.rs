use rust_molkky::{Field, Position, Skittle};

#[test]
fn 指定した番号と位置にスキットルを配置する() {
    let pins = vec![
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(2, 4)),
    ];
    let field = Field::new(pins.clone());
    assert_eq!(field.skittles(), pins);
}

#[test]
#[should_panic(expected = "スキットル番号が重複しています: 5")]
fn 同じ番号は位置が違っても配置できない() {
    Field::new(vec![
        Skittle::new(5),
        Skittle::new(5).with_position(Position::new(3, 4)),
    ]);
}

#[test]
fn 既定の配置には1から12番が異なる位置に並ぶ() {
    use std::collections::HashSet;
    let field = Field::default();
    assert_eq!(
        field
            .skittles()
            .iter()
            .map(Skittle::number)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
    assert_eq!(
        field
            .skittles()
            .iter()
            .map(Skittle::position)
            .collect::<HashSet<_>>()
            .len(),
        12
    );
    assert_eq!(field.skittles()[0].position(), Position::new(-1, 3));
    assert_eq!(field.skittles()[11].position(), Position::new(1, 6));
}

#[test]
fn 同じ座標でも番号が異なるスキットルは配置できる() {
    let field = Field::new(vec![Skittle::new(1), Skittle::new(2)]);
    assert_eq!(field.skittles().len(), 2);
    assert_eq!(
        field.skittles()[0].position(),
        field.skittles()[1].position()
    );
}
