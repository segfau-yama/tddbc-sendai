use rust_molkky::core::models::Stick;
#[test]
fn 座標を使わず確率の境界と既定値を保持する() {
    assert_eq!(Stick::default().fall_probability(), 0.25);
    assert_eq!(Stick::new(0.0).fall_probability(), 0.0);
    assert_eq!(Stick::new(1.0).fall_probability(), 1.0);
}
#[test]
fn 範囲外と非有限の確率を拒否する() {
    for p in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(std::panic::catch_unwind(|| Stick::new(p)).is_err());
    }
}
