use rust_molkky::{Field, gui_2d::models::GuiField};

mod identity {
    use super::*;

    #[test]
    fn 既存フィールドの識別子を再生成せず保持する() {
        let original = Field::new();
        let field = GuiField::new(original);
        assert_eq!(*field.as_ref(), original);
        assert_eq!(field.clone().as_ref().id(), original.id());
    }
}
