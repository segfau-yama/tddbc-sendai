use super::*;
use crate::test_support::ConstantRng;
use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::simulation::{RandomThrow, StandardFieldGenerator, Stick};
use rust_molkky::{Field, FieldGenerator, calc_score, throw_stick};
#[test]
fn 同番号でも倒れた個体だけを移動し元の表示と他の個体を保持する() {
    let pins = [Skittle::new(12), Skittle::new(12)];
    let original = FieldLayout::new(&pins);
    let next = relocate_skittles(&original, &[pins[0]], &mut ConstantRng(0));
    assert_ne!(
        next.positions[&pins[0].id()],
        original.positions[&pins[0].id()]
    );
    assert_eq!(
        next.positions[&pins[1].id()],
        original.positions[&pins[1].id()]
    );
    assert_eq!(original, FieldLayout::new(&pins));
    assert_eq!(next.stick, original.stick);
    assert_eq!(
        relocate_skittles(&original, &[], &mut ConstantRng(0)),
        original
    );
}
#[test]
fn 同じシードなら点数列と表示移動を再現しピン集合は変わらない() {
    let pins = StandardFieldGenerator.generate();
    let original_pins = pins.skittles().to_vec();
    let (mut first, mut second) = (StdRng::seed_from_u64(42), StdRng::seed_from_u64(42));
    let (mut a, mut b) = (
        FieldLayout::new(pins.skittles()),
        FieldLayout::new(pins.skittles()),
    );
    for _ in 0..100 {
        let hit_a =
            throw_stick(&pins, &mut RandomThrow::new(&Stick::default(), &mut first)).unwrap();
        let hit_b =
            throw_stick(&pins, &mut RandomThrow::new(&Stick::default(), &mut second)).unwrap();
        assert_eq!(hit_a.knocked_over(), hit_b.knocked_over());
        assert!((0..=12).contains(&calc_score(hit_a.knocked_over()).value()));
        a = relocate_skittles(&a, hit_a.knocked_over(), &mut first);
        b = relocate_skittles(&b, hit_b.knocked_over(), &mut second);
        assert_eq!(a, b);
        assert_eq!(a.positions.len(), 12);
    }
    assert_eq!(pins.skittles(), original_pins);
}
#[test]
fn 整数の四隅でも倒れたピンを隣接位置へ移せる() {
    let pin = Skittle::new(5);
    for (x, y) in [
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        let mut layout = FieldLayout::new(&[pin]);
        layout.positions.insert(pin.id(), Position::new(x, y));
        let next = relocate_skittles(&layout, &[pin], &mut ConstantRng(u64::MAX));
        let moved = next.positions[&pin.id()];
        assert_ne!(moved, layout.positions[&pin.id()]);
        assert!((i64::from(moved.x()) - i64::from(x)).abs() <= 1);
        assert!((i64::from(moved.y()) - i64::from(y)).abs() <= 1);
        assert_eq!(layout.positions[&pin.id()], Position::new(x, y));
    }
}
#[test]
fn 初期表示は投てき側から二本三本四本三本の順に並ぶ() {
    let field = StandardFieldGenerator.generate();
    let layout = FieldLayout::new(field.skittles());
    for (y, expected) in [
        (3, vec![1, 2]),
        (4, vec![3, 10, 4]),
        (5, vec![5, 11, 12, 6]),
        (6, vec![7, 9, 8]),
    ] {
        let mut row: Vec<_> = field
            .skittles()
            .iter()
            .filter(|pin| layout.positions[&pin.id()].y() == y)
            .collect();
        row.sort_by_key(|pin| layout.positions[&pin.id()].x());
        assert_eq!(
            row.iter().map(|pin| pin.number()).collect::<Vec<_>>(),
            expected
        );
    }
    assert_eq!(layout.positions.len(), 12);
}
