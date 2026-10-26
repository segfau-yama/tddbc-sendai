use super::*;
use crate::test_support::ConstantRng;
use crate::view::position::Position;
use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::core::{Field, Stick, calc_score, throw_stick};
#[test]
fn 倒れたピンだけを移動し元の表示と立ったピンを保持する() {
    let original = FieldLayout::default();
    let next = relocate_skittles(&original, &[Skittle::new(5)], &mut ConstantRng(0));
    assert_ne!(next.positions[&5], original.positions[&5]);
    for (&number, &position) in &original.positions {
        if number != 5 {
            assert_eq!(next.positions[&number], position);
        }
    }
    assert_eq!(original, FieldLayout::default());
    assert_eq!(next.stick, original.stick);
    assert_eq!(
        relocate_skittles(&original, &[], &mut ConstantRng(0)),
        original
    );
}
#[test]
fn 同じシードなら点数列と表示移動を再現しピン集合は変わらない() {
    let pins = Field::default();
    let original_pins = pins.skittles().to_vec();
    let (mut first, mut second) = (StdRng::seed_from_u64(42), StdRng::seed_from_u64(42));
    let (mut a, mut b) = (FieldLayout::default(), FieldLayout::default());
    for _ in 0..100 {
        let hit_a = throw_stick(&Stick::default(), pins.skittles(), &mut first);
        let hit_b = throw_stick(&Stick::default(), pins.skittles(), &mut second);
        assert_eq!(hit_a, hit_b);
        assert!((0..=12).contains(&calc_score(&hit_a).value()));
        a = relocate_skittles(&a, &hit_a, &mut first);
        b = relocate_skittles(&b, &hit_b, &mut second);
        assert_eq!(a, b);
        assert_eq!(a.positions.len(), 12);
    }
    assert_eq!(pins.skittles(), original_pins);
}
#[test]
fn 整数の四隅でも倒れたピンを隣接位置へ移せる() {
    for (x, y) in [
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        let mut layout = FieldLayout::default();
        layout.positions.insert(5, Position::new(x, y));
        let next = relocate_skittles(&layout, &[Skittle::new(5)], &mut ConstantRng(u64::MAX));
        let moved = next.positions[&5];
        assert_ne!(moved, layout.positions[&5]);
        assert!((i64::from(moved.x()) - i64::from(x)).abs() <= 1);
        assert!((i64::from(moved.y()) - i64::from(y)).abs() <= 1);
        assert_eq!(layout.positions[&5], Position::new(x, y));
    }
}
