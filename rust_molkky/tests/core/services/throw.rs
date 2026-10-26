use crate::support::ConstantRng;
use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::core::{
    models::{Field, Skittle, Stick},
    services::{calc_score, throw_stick},
};
#[test]
fn 確率零なら倒れず確率一なら全ピンが倒れる() {
    let field = Field::default();
    assert!(throw_stick(&Stick::new(0.0), field.skittles(), &mut ConstantRng(0)).is_empty());
    let knocked = throw_stick(
        &Stick::new(1.0),
        field.skittles(),
        &mut ConstantRng(u64::MAX),
    );
    assert_eq!(knocked, field.skittles());
    assert_eq!(calc_score(&knocked).value(), 12);
}
#[test]
fn 投てきは入力のピンと棒の条件を変えない() {
    let pins = [Skittle::new(5)];
    let stick = Stick::new(1.0);
    assert_eq!(
        throw_stick(&stick, &pins, &mut StdRng::seed_from_u64(42)),
        pins
    );
    assert_eq!(pins, [Skittle::new(5)]);
    assert_eq!(stick.fall_probability(), 1.0);
}
#[test]
fn 同じピンと確率とシードなら同じ投てきを再現する() {
    let pins = Field::default();
    let mut first = StdRng::seed_from_u64(20261024);
    let mut second = StdRng::seed_from_u64(20261024);
    for _ in 0..100 {
        assert_eq!(
            throw_stick(&Stick::default(), pins.skittles(), &mut first),
            throw_stick(&Stick::default(), pins.skittles(), &mut second)
        );
    }
}
#[test]
fn 同じ乱数でも棒の設定を変えると結果が変わる() {
    let pins = [Skittle::new(5)];
    assert!(throw_stick(&Stick::new(0.2), &pins, &mut ConstantRng(1 << 62)).is_empty());
    assert_eq!(
        throw_stick(&Stick::new(0.5), &pins, &mut ConstantRng(1 << 62)),
        pins
    );
}
#[test]
fn 対象が空なら何も倒れない() {
    assert!(throw_stick(&Stick::default(), &[], &mut ConstantRng(0)).is_empty());
}
#[test]
#[should_panic(expected = "スキットル番号が重複しています: 5")]
fn 同じ番号の投てき対象を拒否する() {
    throw_stick(
        &Stick::default(),
        &[Skittle::new(5); 2],
        &mut ConstantRng(0),
    );
}
