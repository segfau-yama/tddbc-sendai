use rust_molkky::{Name, Team, gui_2d::view::result};

mod display {
    use super::*;

    #[test]
    fn 得点では判定せず渡された勝者を表示する() {
        let winner = Team::new(Name::new("<青>"));
        let html = dioxus_ssr::render_element(result(Some(&winner)));
        assert!(
            ["&lt;青&gt;の勝利", "&#60;青&#62;の勝利"]
                .iter()
                .any(|escaped| html.contains(escaped)),
            "{html}"
        );
        assert!(!html.contains("<青>"));
    }

    #[test]
    fn 勝者がいない結果を表示できる() {
        let html = dioxus_ssr::render_element(result(None));
        assert!(html.contains("勝者なし"));
    }
}
