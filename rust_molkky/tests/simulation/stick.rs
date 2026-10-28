use crate::support::ConstantRng;
use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::simulation::{RandomStick, StandardFieldGenerator};
use rust_molkky::{Skittle, SkittleState, Stick};

#[test]
fn 確率の境界と既定値を保持し棒の識別子を維持する() {
    let stick = Stick::new();
    assert_eq!(RandomStick::default().fall_probability(), 0.25);
    for probability in [0.0, 1.0] {
        let combined = RandomStick::new(stick, probability);
        assert_eq!(combined.fall_probability(), probability);
        assert_eq!(combined.stick().id(), stick.id());
    }
}
#[test]
fn 範囲外と非有限の確率を拒否する() {
    for p in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(std::panic::catch_unwind(|| RandomStick::new(Stick::new(), p)).is_err());
    }
}
#[test]
fn 確率零なら直立を保ち確率一なら全ピンが倒れる() {
    let field = StandardFieldGenerator.generate();
    let zero = RandomStick::new(Stick::new(), 0.0).judge(field.skittles(), &mut ConstantRng(0));
    assert_eq!(zero, field.skittles());
    let all =
        RandomStick::new(Stick::new(), 1.0).judge(field.skittles(), &mut ConstantRng(u64::MAX));
    assert_eq!(all.len(), 12);
    for (original, updated) in field.skittles().iter().zip(&all) {
        assert_eq!(updated.id(), original.id());
        assert_eq!(updated.state(), SkittleState::Fall);
        assert_eq!(original.state(), SkittleState::Stand);
    }
}
#[test]
fn 判定は入力のピンと棒の条件を変えない() {
    let pins = [Skittle::new(5)];
    let stick = RandomStick::new(Stick::new(), 1.0);
    let original = stick;
    let updated = stick.judge(&pins, &mut StdRng::seed_from_u64(42));
    assert_eq!(updated, [pins[0].with_state(SkittleState::Fall)]);
    assert_eq!(pins[0].state(), SkittleState::Stand);
    assert_eq!(stick, original);
}
#[test]
fn 同じピンと確率とシードなら同じ判定を再現する() {
    let field = StandardFieldGenerator.generate();
    let stick = RandomStick::default();
    let (mut a, mut b) = (StdRng::seed_from_u64(42), StdRng::seed_from_u64(42));
    for _ in 0..100 {
        assert_eq!(
            stick.judge(field.skittles(), &mut a),
            stick.judge(field.skittles(), &mut b)
        );
    }
}
#[test]
fn 同じ乱数でも確率設定を変えると結果が変わる() {
    let pins = [Skittle::new(5)];
    let lower = RandomStick::new(Stick::new(), 0.2).judge(&pins, &mut ConstantRng(1 << 62));
    let higher = RandomStick::new(Stick::new(), 0.5).judge(&pins, &mut ConstantRng(1 << 62));
    assert_eq!(lower, pins);
    assert_eq!(higher, [pins[0].with_state(SkittleState::Fall)]);
}
#[test]
fn 対象が空なら結果も空になる() {
    assert!(
        RandomStick::default()
            .judge(&[], &mut ConstantRng(0))
            .is_empty()
    );
}
#[test]
fn 既に倒れた個体は確率零でも倒れた状態を保つ() {
    let fallen = Skittle::new(12).with_state(SkittleState::Fall);
    assert_eq!(
        RandomStick::new(Stick::new(), 0.0).judge(&[fallen], &mut ConstantRng(0)),
        [fallen]
    );
}
