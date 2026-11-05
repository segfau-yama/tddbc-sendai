use rust_molkky::{Skittle, SkittleState, StandUp, cui::models::CuiSkittle};

mod standing_up {
    use super::*;

    #[test]
    fn 立て直しは個体と番号を保って立位へ戻し連鎖確率を消す() {
        let original = CuiSkittle::new(Skittle::new(5).with_state(SkittleState::Fall), 0.8);
        let standing = original.stand_up();
        assert_eq!(
            *standing.as_ref(),
            original.as_ref().with_state(SkittleState::Stand)
        );
        assert_eq!(standing.probability(), 0.0);
        assert_eq!(original.as_ref().state(), SkittleState::Fall);
        assert_eq!(original.probability(), 0.8);
    }
}

mod composition {
    use super::*;

    #[test]
    fn 既存個体の状態と識別子を保ち連鎖確率を合成する() {
        let original = Skittle::new(5).with_state(SkittleState::Fall);
        let skittle = CuiSkittle::new(original, 0.8);
        assert_eq!(skittle.as_ref(), &original);
        assert_eq!(skittle.probability(), 0.8);
        let initial = CuiSkittle::from(Skittle::new(5));
        assert_eq!(initial.as_ref().state(), SkittleState::Stand);
        assert_eq!(initial.probability(), 0.0);
        assert_ne!(initial.as_ref().id(), original.id());
    }
}
