use rust_molkky::{
    Molkky, Name, ScoreBoard, Skittle, Team,
    gui_2d::{models::GuiStick, services::GuiMolkky, view::scores},
};

fn team(name: &str, points: &[i32]) -> Team {
    let rules = GuiMolkky::new(&GuiStick::default(), [0.0, 0.0], &[]);
    let history = points
        .iter()
        .map(|&value| {
            let pins = if value == 0 {
                vec![]
            } else {
                vec![Skittle::new(value)]
            };
            rules.calc_score(&pins)
        })
        .collect();
    Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
}

mod display {
    use super::*;

    #[test]
    fn 現在チームと履歴と合計を表示し名前はエスケープする() {
        let teams = vec![team("<赤&>", &[5, 8]).with_turn(true), team("青", &[])];
        let html = dioxus_ssr::render_element(scores(&teams));
        assert!(
            ["&lt;赤&amp;&gt;", "&#60;赤&#38;&#62;"]
                .iter()
                .any(|escaped| html.contains(escaped)),
            "{html}"
        );
        assert!(!html.contains("<赤&>"));
        assert!(html.contains("aria-current=\"true\""));
        assert!(html.contains("▶"));
        assert!(html.contains(&teams[0].id().to_string()));
        assert!(html.contains("13点"));
        assert!(html.contains("5, 8"));
        assert!(html.contains("青"));
        assert!(html.contains("0点"));
    }

    #[test]
    fn 補正と失格と勝利を再計算せず得点表の状態から表示する() {
        let teams = vec![
            team("赤", &[12, 12, 12, 12, 5]),
            team("青", &[0, 0, 0]),
            team("緑", &[12, 12, 12, 12, 2]),
        ];
        let html = dioxus_ssr::render_element(scores(&teams));
        assert!(html.contains("25点に補正"));
        assert!(html.contains("12, 12, 12, 12, 5"));
        assert!(html.contains("<td>失格</td>"));
        assert!(html.contains("<td>勝利</td>"));
        assert!(html.contains("50点"));
    }
}
