use rust_molkky::{Skittle, SkittleState, cui::models::CuiSkittle};

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
