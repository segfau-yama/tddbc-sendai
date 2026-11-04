use rust_molkky::{Field, Skittle, gui_2d::models::GuiField};

mod identity {
    use super::*;

    #[test]
    fn 既存フィールドと同番号の個体も再生成せず保持する() {
        let core_field = Field::new();
        let skittles = vec![Skittle::new(12), Skittle::new(12)];
        let field = GuiField::new(core_field, skittles.clone());
        assert_eq!(field.as_ref().id(), core_field.id());
        assert_eq!(field.skittles(), skittles);
        assert_ne!(field.skittles()[0].id(), field.skittles()[1].id());
    }
}
