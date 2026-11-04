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

mod updating {
    use super::*;

    #[test]
    fn 物理結果から同じ個体を更新し元の棒と目標を保つ() {
        let original = GuiStick::throw(GuiStick::new(Stick::new(), Some([0.0, 1.0]), 6.0));
        let updated = original.update([0.1, 0.5], 0.25, [3.0, 4.0], 0.5);
        assert_eq!(updated.as_ref().id(), original.as_ref().id());
        assert_eq!(updated.target(), original.target());
        assert_eq!(updated.position(), [0.1, 0.5]);
        assert_eq!(updated.angle(), 0.25);
        assert_eq!(updated.velocity(), [3.0, 4.0]);
        assert_eq!(updated.angular_velocity(), 0.5);
        assert_eq!(updated.speed(), 5.0);
        assert_eq!(updated.as_ref().state(), StickState::Move);
        assert_eq!(original.position(), [0.0; 2]);
        assert_eq!(original.speed(), 6.0);
    }

    #[test]
    fn 並進と回転が両方止まったときだけ停止状態になる() {
        let original = GuiStick::throw(GuiStick::new(Stick::new(), Some([0.0, 1.0]), 6.0));
        let rotating = original.update([0.0, 0.5], 0.25, [0.0; 2], 0.5);
        assert_eq!(rotating.as_ref().state(), StickState::Move);
        let stopped = rotating.update([0.0, 0.5], 0.5, [0.0; 2], 0.0);
        assert_eq!(stopped.as_ref().state(), StickState::Stop);
        assert_eq!(stopped.speed(), 0.0);
        assert_eq!(stopped.position(), [0.0, 0.5]);
        assert_eq!(rotating.as_ref().state(), StickState::Move);
        assert_eq!(
            stopped.update(stopped.position(), stopped.angle(), [0.0; 2], 0.0),
            stopped
        );
    }
}
