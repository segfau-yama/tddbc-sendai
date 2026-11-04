use super::*;
use crate::gui_2d::{controllers::pose, services::field::create_world};
use crate::{Molkky, Stick, Throw};

mod next_throw {
    use super::*;

    #[test]
    fn 次投の飛行が始まるまでは前回の停止位置でなく投擲位置に表示する() {
        let stick = GuiStick::new(Stick::new(), Some([0.0, 1.0]), 0.0).throw(6.0);
        let mut rules = create_world(&stick, GuiField::THROW_POSITION, &[]);
        assert!((0..1200).any(|_| !rules.fly(&stick)));
        assert_ne!(
            pose(&rules, stick.stick().id()).unwrap().0,
            GuiField::THROW_POSITION
        );
        let held = dioxus_ssr::render_element(render(&stick, &rules, false));
        assert_eq!(
            dioxus_ssr::render_element(render(&stick, &rules, true)),
            held
        );
        assert!(rules.fly(&stick));
        assert_ne!(
            dioxus_ssr::render_element(render(&stick, &rules, true)),
            held
        );
    }
}
