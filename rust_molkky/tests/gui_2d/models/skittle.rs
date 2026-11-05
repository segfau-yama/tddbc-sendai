use rust_molkky::{Skittle, SkittleState, StandUp, gui_2d::models::GuiSkittle};

mod standing_up {
    use super::*;

    #[test]
    fn 立て直しは個体と番号と移動後の位置を保ち角度と照準を戻す() {
        let original = GuiSkittle::new(Skittle::new(12), [0.1, 0.7]).update(
            [0.2, 0.9],
            1.2,
            SkittleState::Fall,
            true,
        );
        let standing = original.stand_up();
        assert_eq!(
            *standing.as_ref(),
            original.as_ref().with_state(SkittleState::Stand)
        );
        assert_eq!(standing.position(), original.position());
        assert_eq!(standing.angle(), 0.0);
        assert!(!standing.aimed());
        assert_eq!(original.angle(), 1.2);
        assert!(original.aimed());
        assert_eq!(original.as_ref().state(), SkittleState::Fall);
    }
}

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
