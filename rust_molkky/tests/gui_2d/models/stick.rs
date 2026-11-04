use rust_molkky::{Stick, Throw, gui_2d::models::GuiStick};

mod throwing {
    use super::*;

    #[test]
    fn 速度だけを更新し照準と個体を保ち元の棒を変更しない() {
        let original = GuiStick::new(Stick::new(), Some([1.5, 3.0]), 2.0);
        let updated = original.throw(6.0);
        assert_eq!(updated.speed(), 6.0);
        assert_eq!(updated.target(), original.target());
        assert_eq!(updated.as_ref(), original.as_ref());
        assert_eq!(original.speed(), 2.0);
    }
}

mod aiming {
    use super::*;

    #[test]
    fn 未照準から同じ棒に目標と確率ではない初速を設定できる() {
        let initial = GuiStick::default();
        assert_eq!(initial.target(), None);
        assert_eq!(initial.speed(), 0.0);

        let core_stick: Stick = *initial.stick();
        let aimed = GuiStick::new(core_stick, Some([1.5, 3.0]), 6.0);
        assert_eq!(aimed.stick().id(), initial.stick().id());
        assert_eq!(aimed.as_ref(), &core_stick);
        assert_eq!(aimed.target(), Some([1.5, 3.0]));
        assert_eq!(aimed.speed(), 6.0);
        assert_eq!(initial.target(), None);
        assert_eq!(initial.speed(), 0.0);
    }
}
