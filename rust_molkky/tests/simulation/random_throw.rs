use crate::support::ConstantRng;
use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::simulation::{ArrayField, RandomThrow, StandardFieldGenerator, Stick};
use rust_molkky::{Field, FieldGenerator, Skittle, calc_score, throw_stick};

#[test]
fn 確率零なら倒れず確率一なら全ピンが倒れる() {
    let field = StandardFieldGenerator.generate();
    let zero = throw_stick(
        &field,
        &mut RandomThrow::new(&Stick::new(0.0), &mut ConstantRng(0)),
    )
    .unwrap();
    assert!(zero.knocked_over().is_empty());
    let all = throw_stick(
        &field,
        &mut RandomThrow::new(&Stick::new(1.0), &mut ConstantRng(u64::MAX)),
    )
    .unwrap();
    assert_eq!(all.knocked_over(), field.skittles());
    assert_eq!(calc_score(all.knocked_over()).value(), 12);
}
#[test]
fn 投てきは入力のピンと棒の条件を変えない() {
    let pin = Skittle::new(5);
    let field = ArrayField::new(vec![pin]);
    let stick = Stick::new(1.0);
    let outcome = throw_stick(
        &field,
        &mut RandomThrow::new(&stick, &mut StdRng::seed_from_u64(42)),
    )
    .unwrap();
    assert_eq!(outcome.knocked_over(), [pin]);
    assert_eq!(field.skittles(), [pin]);
    assert_eq!(outcome.field().skittles(), [pin]);
    assert_eq!(stick.fall_probability(), 1.0);
}
#[test]
fn 同じピンと確率とシードなら同じ投てきを再現する() {
    let field = StandardFieldGenerator.generate();
    let (mut a, mut b) = (StdRng::seed_from_u64(42), StdRng::seed_from_u64(42));
    for _ in 0..100 {
        assert_eq!(
            throw_stick(&field, &mut RandomThrow::new(&Stick::default(), &mut a))
                .unwrap()
                .knocked_over(),
            throw_stick(&field, &mut RandomThrow::new(&Stick::default(), &mut b))
                .unwrap()
                .knocked_over()
        );
    }
}
#[test]
fn 同じ乱数でも棒の設定を変えると結果が変わる() {
    let field = ArrayField::new(vec![Skittle::new(5)]);
    assert!(
        throw_stick(
            &field,
            &mut RandomThrow::new(&Stick::new(0.2), &mut ConstantRng(1 << 62))
        )
        .unwrap()
        .knocked_over()
        .is_empty()
    );
    assert_eq!(
        throw_stick(
            &field,
            &mut RandomThrow::new(&Stick::new(0.5), &mut ConstantRng(1 << 62))
        )
        .unwrap()
        .knocked_over(),
        field.skittles()
    );
}
#[test]
fn 対象が空なら何も倒れない() {
    assert!(
        throw_stick(
            &ArrayField::new(vec![]),
            &mut RandomThrow::new(&Stick::default(), &mut ConstantRng(0))
        )
        .unwrap()
        .knocked_over()
        .is_empty()
    );
}
