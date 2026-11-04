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
                .all(|skittle| skittle.state() == SkittleState::Stand)
        );
    }
}

mod identity {
    use super::*;

    #[test]
    fn 既存フィールドと同番号の個体も再生成せず保持する() {
        let core_field = Field::new();
        let skittles = vec![Skittle::new(12), Skittle::new(12)];
        let field = GuiField::new(core_field, skittles.clone());
        assert_eq!(field.field().id(), core_field.id());
        assert_eq!(field.as_ref().id(), core_field.id());
        assert_eq!(field.skittles(), skittles);
        assert_ne!(field.skittles()[0].id(), field.skittles()[1].id());
    }
}

mod initial_positions {
    use super::*;

    #[test]
    fn 同じ個体を標準の四列へ配置し投擲位置から一メートル離す() {
        let field = GuiField::default();
        let positions = field.initial_positions(0.0275);
        let numbers: Vec<_> = positions
            .iter()
            .map(|(id, _)| {
                field
                    .skittles()
                    .iter()
                    .find(|skittle| skittle.id() == *id)
                    .unwrap()
                    .number()
            })
            .collect();
        assert_eq!(numbers, [1, 2, 3, 10, 4, 5, 11, 12, 6, 7, 9, 8]);
        assert_eq!(
            positions.iter().map(|(id, _)| *id).collect::<HashSet<_>>(),
            field.skittles().iter().map(Skittle::id).collect()
        );
        for ((_, [x, y]), [expected_x, expected_y]) in positions.iter().zip([
            [-0.0275, 1.0],
            [0.0275, 1.0],
            [-0.055, 1.0476314],
            [0.0, 1.0476314],
            [0.055, 1.0476314],
            [-0.0825, 1.0952628],
            [-0.0275, 1.0952628],
            [0.0275, 1.0952628],
            [0.0825, 1.0952628],
            [-0.055, 1.1428941],
            [0.0, 1.1428941],
            [0.055, 1.1428941],
        ]) {
            assert!((x - expected_x).abs() < 0.000001);
            assert!((y - expected_y).abs() < 0.000001);
        }
        assert_eq!(GuiField::THROW_POSITION, [0.0, 0.0]);
        assert_eq!(positions, field.initial_positions(0.0275));
    }
}
