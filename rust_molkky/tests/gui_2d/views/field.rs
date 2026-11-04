use dioxus::prelude::*;
use rust_molkky::gui_2d::views::field;

mod composition {
    use super::*;

    #[test]
    fn 範囲を保って上向きの座標と子要素を表示する() {
        let html = dioxus_ssr::render_element(field(
            [-2.0, 0.0, 4.0, 8.0],
            rsx! { circle { cx: "1", cy: "2", r: "0.1" } },
        ));
        assert!(html.contains("viewBox=\"-2 -8 4 8\""), "{html}");
        assert!(html.contains("scale(1 -1)"));
        assert!(html.contains("<circle"));
        assert!(html.contains("モルックのフィールド"));
    }
}
