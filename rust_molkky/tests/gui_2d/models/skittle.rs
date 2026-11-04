use rust_molkky::{Skittle, SkittleState, gui_2d::models::GuiSkittle};

mod updating {
    use super::*;

    #[test]
    fn 自分の物理結果と判定を反映し個体と番号と元の値を保つ() {
        let original = GuiSkittle::new(Skittle::new(12), [0.0, 1.0]);
        let updated = original.update([0.1, 0.7], 0.25, SkittleState::Hit, true);
        assert_eq!(updated.as_ref().id(), original.as_ref().id());
        assert_eq!(updated.as_ref().number(), 12);
        assert_eq!(updated.as_ref().state(), SkittleState::Hit);
        assert_eq!(updated.position(), [0.1, 0.7]);
        assert_eq!(updated.angle(), 0.25);
        assert!(updated.aimed());
        let fallen = updated.update(
            updated.position(),
            updated.angle(),
            SkittleState::Fall,
            false,
        );
        assert_eq!(fallen.as_ref().state(), SkittleState::Fall);
        assert_eq!(fallen.as_ref().id(), original.as_ref().id());
        assert!(!fallen.aimed());
        assert_eq!(original.position(), [0.0, 1.0]);
        assert_eq!(original.angle(), 0.0);
        assert_eq!(original.as_ref().state(), SkittleState::Stand);
        assert!(!original.aimed());
        assert_eq!(updated.as_ref().state(), SkittleState::Hit);
    }
}
