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
                .map(|skittle| skittle.number())
                .collect::<Vec<_>>(),
            (1..=12).collect::<Vec<_>>()
        );
        assert_eq!(
            field
                .skittles
                .iter()
                .map(|skittle| skittle.id())
                .collect::<HashSet<_>>()
                .len(),
            12
        );
        assert!(
            field
                .skittles
                .iter()
                .all(|skittle| skittle.state() == SkittleState::Stand)
        );
    }
}
