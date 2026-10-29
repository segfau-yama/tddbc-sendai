use rust_molkky::{SkittleState, cui::models::Field};
use std::collections::HashSet;

mod initialization {
    use super::*;

    #[test]
    fn 一から十二の番号を持つ別個体を立った状態で用意する() {
        let field = Field::default();
        assert_eq!(
            field
                .skittles
                .iter()
                .map(|pin| pin.number())
                .collect::<Vec<_>>(),
            (1..=12).collect::<Vec<_>>()
        );
        assert_eq!(
            field
                .skittles
                .iter()
                .map(|pin| pin.id())
                .collect::<HashSet<_>>()
                .len(),
            12
        );
        assert!(
            field
                .skittles
                .iter()
                .all(|pin| pin.state() == SkittleState::Stand)
        );
    }
}
