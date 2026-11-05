use dioxus::prelude::*;
use rust_molkky::{
    Skittle, SkittleState,
    gui_2d::{models::GuiSkittle, views::Skittle as SkittleView},
};

mod appearance {
    use super::*;
    #[test]
    fn モデルの番号と配置を保ち立位と命中中と転倒を描き分ける() {
        let skittle = Skittle::new(12);
        let mut htmls = Vec::new();
        for (state, tag) in [
            (SkittleState::Stand, "stand"),
            (SkittleState::Hit, "hit"),
            (SkittleState::Fall, "fall"),
        ] {
            let model = GuiSkittle::new(skittle.with_state(state), [1.0, 2.0]);
            let html = dioxus_ssr::render_element(dioxus::prelude::rsx! { SkittleView { model } });
            assert!(html.contains(&format!("skittle-{}", skittle.id())));
            assert!(html.contains("translate(1 2)"));
            assert!(html.contains(">12</text>"));
            assert!(html.contains(&format!("data-state=\"{tag}\"")));
            htmls.push(html);
        }
        assert_ne!(htmls[0], htmls[1]);
        assert_ne!(htmls[1], htmls[2]);
    }
}
