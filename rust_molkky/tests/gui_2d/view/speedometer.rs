use rust_molkky::gui_2d::view::speedometer;

mod display {
    use super::*;

    #[test]
    fn 速度と上限を物理単位で表示する() {
        let html = dioxus_ssr::render_element(speedometer(6.0, 10.0));
        assert!(html.contains("<meter"));
        assert!(html.contains("value=\"6\""));
        assert!(html.contains("max=\"10\""));
        assert!(html.contains("6.0 m/s"));
        assert!(html.contains("投擲速度"));
    }
}
