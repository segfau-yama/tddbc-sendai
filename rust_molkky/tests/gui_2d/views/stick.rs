use rust_molkky::{
    Stick, StickState, Throw,
    gui_2d::{
        models::{GuiField, GuiStick},
        services::field::create_world,
        views::stick,
    },
};

mod geometry {
    use super::*;
    #[test]
    fn モデルの識別子とラジアン角と寸法で棒を描く() {
        let model = GuiStick::throw(GuiStick::new(Stick::new(), Some([1.0, 0.0]), 6.0));
        let html = dioxus_ssr::render_element(stick(&model));
        assert!(html.contains(&format!("stick-{}", model.as_ref().id())));
        assert!(html.contains("translate(0 0) rotate(-90)"), "{html}");
        assert!(html.contains(&format!("width=\"{}\"", 2.0 * GuiStick::RADIUS)));
        assert!(html.contains(&format!("height=\"{}\"", GuiStick::LENGTH)));
    }
}

mod next_throw {
    use super::*;
    #[test]
    fn 初期化した次投のモデルは物理更新前も投擲位置で表示される() {
        let mut model = GuiStick::throw(GuiStick::new(Stick::new(), Some([0.0, 1.0]), 6.0));
        let mut skittles = vec![];
        let mut rules = create_world(&model, &skittles);
        for _ in 0..1200 {
            (skittles, model) = rules.update(&skittles, model);
            if model.as_ref().state() == StickState::Stop {
                break;
            }
        }
        assert_eq!(model.as_ref().state(), StickState::Stop);
        assert_ne!(model.position(), GuiField::THROW_POSITION);
        let held = GuiStick::new(*model.as_ref(), Some([0.0, 1.0]), 6.0);
        let html = dioxus_ssr::render_element(stick(&held));
        assert!(html.contains("translate(0 0) rotate(0)"), "{html}");
        let thrown = GuiStick::throw(held);
        assert_eq!(thrown.position(), GuiField::THROW_POSITION);
        assert_eq!(dioxus_ssr::render_element(stick(&thrown)), html);
        let (_, moved) = rules.update(&skittles, thrown);
        assert_ne!(dioxus_ssr::render_element(stick(&moved)), html);
    }
}
