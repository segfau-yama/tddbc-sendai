use rust_molkky::{
    Skittle, SkittleState, StickState,
    gui_2d::{
        models::{GuiField, GuiSkittle, GuiStick},
        services::field::{create_field, create_world},
    },
};
use std::collections::HashSet;

mod initialization {
    use super::*;

    #[test]
    fn 不正な配置を物理ワールドへ渡さない() {
        let stick = GuiStick::default();
        let skittle = Skittle::new(12);
        for skittles in [
            vec![
                GuiSkittle::new(skittle, [0.0, 1.0]),
                GuiSkittle::new(skittle, [0.1, 1.0]),
            ],
            vec![GuiSkittle::new(skittle, [f32::INFINITY, 1.0])],
        ] {
            assert!(std::panic::catch_unwind(|| create_world(&stick, &skittles)).is_err());
        }
    }

    #[test]
    fn 通常フィールドには一から十二の別個体が立っている() {
        let (_, stick, skittles) = create_field();
        assert_eq!(
            skittles
                .iter()
                .map(|skittle| skittle.as_ref().number())
                .collect::<Vec<_>>(),
            (1..=12).collect::<Vec<_>>()
        );
        assert_eq!(
            skittles
                .iter()
                .map(|skittle| skittle.as_ref().id())
                .collect::<HashSet<_>>()
                .len(),
            12
        );
        assert!(
            skittles
                .iter()
                .all(|s| s.as_ref().state() == SkittleState::Stand)
        );
        assert_eq!(stick.target(), None);
        assert_eq!(stick.speed(), 0.0);
        assert_eq!(stick.position(), GuiField::THROW_POSITION);
        assert_eq!(stick.as_ref().state(), StickState::Held);
    }

    #[test]
    fn 同じ個体を標準の四列へ配置し投擲位置から一メートル離す() {
        let (_, _, skittles) = create_field();
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
            let skittle = skittles
                .iter()
                .find(|s| s.as_ref().number() == number)
                .unwrap();
            let (position, angle) = (skittle.position(), skittle.angle());
            assert!((position[0] - expected[0]).abs() < 0.000001);
            assert!((position[1] - expected[1]).abs() < 0.000001);
            assert_eq!(angle, 0.0);
            assert_eq!(skittle.position(), position);
        }
        assert_eq!(GuiField::THROW_POSITION, [0.0, 0.0]);
    }
}
