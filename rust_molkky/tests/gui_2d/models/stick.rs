use rust_molkky::{Move, Stick, StickState, Throw, gui_2d::models::GuiStick};

mod throwing {
    use super::*;
    #[test]
    fn 棒だけを受けて目標と強さから速度を生成し同じ個体の移動状態を返す() {
        let original = GuiStick::new(Stick::new(), Some([3.0, 4.0]), 10.0);
        let thrown = GuiStick::throw(original);
        assert_eq!(thrown.velocity(), [6.0, 8.0]);
        assert_eq!(thrown.target(), original.target());
        assert_eq!(thrown.as_ref().id(), original.as_ref().id());
        assert_eq!(thrown.as_ref().state(), StickState::Move);
        assert_eq!(original.velocity(), [0.0; 2]);
        assert_eq!(original.as_ref().state(), StickState::Held);
        assert_eq!(GuiStick::r#move(thrown), thrown);
    }
}

mod initialization {
    use super::*;
    #[test]
    fn 同じ個体を保持位置と零速度へ初期化し入力した強さを持つ() {
        let initial = GuiStick::default();
        assert_eq!(initial.target(), None);
        assert_eq!(initial.speed(), 0.0);
        let prepared = GuiStick::new(*initial.as_ref(), Some([1.5, 3.0]), 6.0);
        assert_eq!(prepared.as_ref().id(), initial.as_ref().id());
        assert_eq!(prepared.target(), Some([1.5, 3.0]));
        assert_eq!(prepared.speed(), 6.0);
        assert_eq!(prepared.position(), [0.0; 2]);
        assert_eq!(prepared.velocity(), [0.0; 2]);
        assert_eq!(prepared.angle(), 0.0);
        assert_eq!(prepared.as_ref().state(), StickState::Held);
        assert_eq!(GuiStick::r#move(prepared), prepared);
        assert_eq!(initial.target(), None);
    }
}
