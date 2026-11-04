use rust_molkky::{
    Field, Skittle,
    gui_2d::models::{GuiField, GuiSkittle},
};

mod identity {
    use super::*;

    #[test]
    fn 既存フィールドと同番号の個体も再生成せず保持する() {
        let core_field = Field::new();
        let skittles = vec![
            GuiSkittle::new(Skittle::new(12), [0.0, 0.7]),
            GuiSkittle::new(Skittle::new(12), [0.1, 0.7]),
        ];
        let field = GuiField::new(core_field, skittles.clone());
        assert_eq!(field.as_ref().id(), core_field.id());
        assert_eq!(field.skittles(), skittles);
        assert_ne!(
            field.skittles()[0].as_ref().id(),
            field.skittles()[1].as_ref().id()
        );
    }
}
