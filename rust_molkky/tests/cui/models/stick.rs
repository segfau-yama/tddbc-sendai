use rust_molkky::{Move, Stick, StickState, Throw, cui::models::CuiStick};

mod throwing {
    use super::*;
    #[test]
    fn 棒だけを受けて確率を生成した移動状態を返し元の個体を保つ() {
        let original = CuiStick::new(Stick::new(), 0.5);
        let thrown = CuiStick::throw(original);
        assert!((0.0..=1.0).contains(&thrown.probability()));
        assert_eq!(thrown.as_ref().id(), original.as_ref().id());
        assert_eq!(thrown.as_ref().state(), StickState::Move);
        assert_eq!(original.probability(), 0.5);
        assert_eq!(original.as_ref().state(), StickState::Held);
        let stopped = CuiStick::r#move(thrown);
        assert_eq!(stopped.as_ref().state(), StickState::Stop);
        assert_eq!(stopped.as_ref().id(), thrown.as_ref().id());
        assert_eq!(stopped.probability(), thrown.probability());
        assert_eq!(CuiStick::r#move(stopped), stopped);
        assert_eq!(CuiStick::r#move(original), original);
    }
}

mod construction {
    use super::*;
    #[test]
    fn 元の棒と指定した確率を保持する() {
        let original = Stick::new();
        let stick = CuiStick::new(original, 0.25);
        assert_eq!(stick.as_ref().id(), original.id());
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
