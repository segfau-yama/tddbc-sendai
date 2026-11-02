use rust_molkky::{Skittle, SkittleState, gui_2d::view::skittle};

mod appearance {
    use super::*;

    #[test]
    fn 同じ個体の番号と配置を保ち立位と転倒を描き分ける() {
        let pin = Skittle::new(12);
        let standing = dioxus_ssr::render_element(skittle(&pin, [1.0, 2.0], 0.1));
        let fallen = dioxus_ssr::render_element(skittle(
            &pin.with_state(SkittleState::Fall),
            [1.0, 2.0],
            0.1,
        ));
        for html in [&standing, &fallen] {
            assert!(html.contains(&format!("skittle-{}", pin.id())));
            assert!(html.contains("translate(1 2)"));
            assert!(html.contains(">12</text>"), "{html}");
        }
        assert!(standing.contains("data-state=\"stand\""));
        assert!(fallen.contains("data-state=\"fall\""));
        assert_ne!(standing, fallen);
    }
}
