use rust_molkky::{Stick, StickState};

mod state {
    use super::*;

    #[test]
    fn 保持から移動と停止へ更新しても識別子と元の状態を保つ() {
        let held = Stick::new();
        let moved = held.with_state(StickState::Move);
        let stopped = moved.with_state(StickState::Stop);
        assert_eq!(held.state(), StickState::Held);
        assert_eq!(moved.state(), StickState::Move);
        assert_eq!(stopped.state(), StickState::Stop);
        assert_eq!(held.id(), moved.id());
        assert_eq!(held.id(), stopped.id());
    }
}

mod identity {
    use super::*;

    #[test]
    fn 新しい個体は異なる識別子を持ちコピーは同じ識別子を保つ() {
        let first = Stick::new();
        let second = Stick::default();
        assert_ne!(first.id(), second.id());
        let copied = first;
        assert_eq!(copied.id(), first.id());
    }
}
