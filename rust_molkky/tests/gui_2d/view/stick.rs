use rust_molkky::{Stick, gui_2d::view::stick};

mod geometry {
    use super::*;

    #[test]
    fn 渡された位置とラジアン角と寸法で棒を描く() {
        let core = Stick::new();
        let html = dioxus_ssr::render_element(stick(
            core.id(),
            [1.0, 2.0],
            std::f32::consts::FRAC_PI_2,
            0.5,
            0.1,
        ));
        assert!(html.contains(&format!("stick-{}", core.id())));
        assert!(html.contains("translate(1 2) rotate(90)"), "{html}");
        assert!(html.contains("width=\"0.1\""));
        assert!(html.contains("height=\"0.5\""));
    }
}
