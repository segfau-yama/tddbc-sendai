use rust_molkky::{Stick, cui::models::CuiStick};

mod construction {
    use super::*;

    #[test]
    fn 元の棒と指定した確率を保持する() {
        let original = Stick::new();
        let stick = CuiStick::new(original, 0.25);
        assert_eq!(stick.stick().id(), original.id());
        assert_eq!(stick.probability(), 0.25);
    }

    #[test]
    fn 通常の棒の確率は五十パーセントになる() {
        assert_eq!(CuiStick::default().probability(), 0.5);
    }

    #[test]
    fn 零と一を許可し範囲外や非数値の確率を拒否する() {
        for probability in [0.0, 1.0] {
            assert_eq!(
                CuiStick::new(Stick::new(), probability).probability(),
                probability
            );
        }
        for probability in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            assert!(std::panic::catch_unwind(|| CuiStick::new(Stick::new(), probability)).is_err());
        }
    }
}
