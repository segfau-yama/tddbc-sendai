use rust_molkky::gui_2d::{
    models::{GuiSkittle, GuiStick},
    services::{field::create_world, physics},
};
use rust_molkky::{Molkky, Skittle, SkittleState, StickState, Throw};

mod contacts {
    use super::*;

    #[test]
    fn 接触した実個体だけを記録し状態はhitが更新する() {
        let held = GuiStick::default();
        let original = GuiSkittle::new(Skittle::new(12), [0.0, 0.7]);
        let unrelated = GuiSkittle::new(Skittle::new(12), [0.2, 1.0]);
        let mut skittles = vec![original, unrelated];
        let mut stick = GuiStick::throw(GuiStick::new(*held.as_ref(), Some([0.0, 1.0]), 6.0));
        let mut rules = create_world(&held, &skittles);
        for _ in 0..1200 {
            (skittles, stick) = physics::step(&mut rules, &skittles, stick);
            if !physics::contacts(&rules).is_empty() {
                break;
            }
        }
        let contacts = physics::contacts(&rules);
        assert_eq!(contacts.len(), 1);
        let (a, b) = contacts[0];
        assert!(
            a == stick.as_ref().id() && b == original.as_ref().id()
                || b == stick.as_ref().id() && a == original.as_ref().id()
        );
        assert_ne!(skittles[0].position(), original.position());
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Stand);
        let hit = rules.hit(&stick, &skittles[0]);
        assert_eq!(hit.as_ref().state(), SkittleState::Hit);
        assert_eq!(skittles[1], unrelated);
        assert_eq!(original.as_ref().state(), SkittleState::Stand);

        // 接触が終わった後も、停止時の判定に同じ履歴を使える。
        for _ in 0..1200 {
            (skittles, stick) = physics::step(&mut rules, &skittles, stick);
            skittles[0] = rules.hit(&stick, &skittles[0]);
            if stick.as_ref().state() == StickState::Stop
                && skittles[0].as_ref().state() != SkittleState::Hit
            {
                break;
            }
        }
        assert_eq!(stick.as_ref().state(), StickState::Stop);
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Fall);
        assert_eq!(physics::contacts(&rules).len(), 1);
        skittles[0] = rules.stand_up(&skittles[0]);
        assert!(physics::contacts(&rules).is_empty());
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Stand);
    }
}
