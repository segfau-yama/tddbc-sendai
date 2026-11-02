use rust_molkky::{Field, Skittle, SkittleState, gui_2d::models::GuiField};
use std::collections::HashSet;

mod initialization {
    use super::*;

    #[test]
    fn 通常フィールドには一から十二の別個体が立っている() {
        let field = GuiField::default();
        assert_eq!(
            field
                .skittles()
                .iter()
                .map(Skittle::number)
                .collect::<Vec<_>>(),
            (1..=12).collect::<Vec<_>>()
        );
        assert_eq!(
            field
                .skittles()
                .iter()
                .map(Skittle::id)
                .collect::<HashSet<_>>()
                .len(),
            12
        );
        assert!(
            field
                .skittles()
                .iter()
                .all(|pin| pin.state() == SkittleState::Stand)
        );
    }
}

mod identity {
    use super::*;

    #[test]
    fn 既存フィールドと同番号の個体も再生成せず保持する() {
        let core_field = Field::new();
        let pins = vec![Skittle::new(12), Skittle::new(12)];
        let field = GuiField::new(core_field, pins.clone());
        assert_eq!(field.field().id(), core_field.id());
        assert_eq!(field.skittles(), pins);
        assert_ne!(field.skittles()[0].id(), field.skittles()[1].id());
    }
}
