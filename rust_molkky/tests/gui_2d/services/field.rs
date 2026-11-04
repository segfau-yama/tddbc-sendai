use rust_molkky::{
    Skittle, SkittleState,
    gui_2d::{models::GuiField, services::field::create_field},
};
use std::collections::HashSet;

mod initialization {
    use super::*;

    #[test]
    fn 通常フィールドには一から十二の別個体が立っている() {
        let (field, stick, rules) = create_field();
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
                .all(|s| s.state() == SkittleState::Stand)
        );
        assert_eq!(stick.target(), None);
        assert_eq!(stick.speed(), 0.0);
        assert_eq!(
            rules.pose(stick.stick().id()).unwrap().0,
            GuiField::THROW_POSITION
        );
        assert!(rules.is_settled());
    }

    #[test]
    fn 同じ個体を標準の四列へ配置し投擲位置から一メートル離す() {
        let (field, _, rules) = create_field();
        for (number, expected) in [
            (1, [-0.0275, 1.0]),
            (2, [0.0275, 1.0]),
            (3, [-0.055, 1.0476314]),
            (10, [0.0, 1.0476314]),
            (4, [0.055, 1.0476314]),
            (5, [-0.0825, 1.0952628]),
            (11, [-0.0275, 1.0952628]),
            (12, [0.0275, 1.0952628]),
            (6, [0.0825, 1.0952628]),
            (7, [-0.055, 1.1428941]),
            (9, [0.0, 1.1428941]),
            (8, [0.055, 1.1428941]),
        ] {
            let skittle = field
                .skittles()
                .iter()
                .find(|s| s.number() == number)
                .unwrap();
            let (position, angle) = rules.pose(skittle.id()).unwrap();
            assert!((position[0] - expected[0]).abs() < 0.000001);
            assert!((position[1] - expected[1]).abs() < 0.000001);
            assert_eq!(angle, 0.0);
            assert_eq!(rules.pose(skittle.id()).unwrap().0, position);
        }
        assert_eq!(GuiField::THROW_POSITION, [0.0, 0.0]);
    }
}
