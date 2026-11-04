use rust_molkky::{Stick, gui_2d::models::GuiStick};

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
