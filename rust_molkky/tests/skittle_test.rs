use rust_molkky::{Position, Skittle, Throw};

#[test]
fn 番号1を保持する() {
    assert_eq!(Skittle::new(1).number(), 1);
}

#[test]
fn 番号12を保持する() {
    assert_eq!(Skittle::new(12).number(), 12);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 番号0のスキットルは生成時に拒否する() {
    Skittle::new(0);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 番号13のスキットルは生成時に拒否する() {
    Skittle::new(13);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 複数本に範囲外の番号が含まれると生成時に拒否する() {
    Throw::new(&[Skittle::new(1), Skittle::new(13)]);
}

#[test]
fn 位置を指定した新しいスキットルを作り元の位置は変えない() {
    let original = Skittle::new(5);
    let moved = original.with_position(Position::new(3, 4));
    assert_eq!(original.position(), Position::new(0, 0));
    assert_eq!(moved.position(), Position::new(3, 4));
    assert_eq!(moved.number(), 5);
}

#[test]
fn 投てき位置から遠いほど倒れる確率が低くなる() {
    let pin = Skittle::new(5).with_position(Position::new(3, 4));
    assert_eq!(pin.fall_probability(Position::new(3, 4)), 1.0);
    assert!((pin.fall_probability(Position::new(0, 0)) - 1.0 / 6.0).abs() < 1e-12);
    assert!(
        pin.fall_probability(Position::new(-3, -4)) < pin.fall_probability(Position::new(0, 0))
    );
}
