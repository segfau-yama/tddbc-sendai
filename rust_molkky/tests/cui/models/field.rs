use rust_molkky::{Field, Skittle, SkittleState, cui::models::CuiField};
use std::collections::HashSet;

mod initialization {
    use super::*;

    #[test]
    fn 既存のフィールドと個体を生成し直さず委譲する() {
        let core_field = Field::new();
        let skittles = vec![Skittle::new(5), Skittle::new(5)];
        let field = CuiField::new(core_field, skittles.clone());
        assert_eq!(field.as_ref().id(), core_field.id());
        assert_eq!(field.skittles, skittles);
    }

    #[test]
    fn 一から十二の番号を持つ別個体を立った状態で用意する() {
        let field = CuiField::default();
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
