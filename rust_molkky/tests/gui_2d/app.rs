use super::*;
use crate::{
    Skittle,
    gui_2d::{controller::PointerSample, models::GuiStick},
};

fn team(name: &str, points: &[i32]) -> Team {
    let stick = GuiStick::default();
    let rules = GuiMolkky::new(&stick, GuiField::THROW_POSITION, &[]);
    let history = points
        .iter()
        .map(|&value| {
            let skittles = if value == 0 {
                vec![]
            } else {
                vec![Skittle::new(value)]
            };
            rules.calc_score(&skittles)
        })
        .collect();
    Team::new(Name::new(name)).with_score_board(crate::ScoreBoard::new(history))
}

fn pointer(target: [f64; 2], time: f64) -> PointerSample {
    PointerSample {
        pointer_id: 1,
        button: 0,
        cursor: [(target[0] + 1.0) * 100.0, (3.0 - target[1]) * 100.0],
        rect: [0.0, 0.0, 200.0, 400.0],
        bounds: [-1.0, -1.0, 2.0, 4.0],
        time,
    }
}

mod composition {
    use super::*;

    #[test]
    fn 最初の手番と十二本と得点表を描画する() {
        let state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        assert!(state.game().teams[0].turn());
        assert!(!state.game().teams[1].turn());
        let html = dioxus_ssr::render_element(render(&state));
        assert_eq!(html.matches("data-state=\"stand\"").count(), 12);
        assert!(html.contains("data-phase=\"ready\""));
        assert!(html.contains("得点表"));
        assert!(html.contains("<meter"));
        assert!(html.contains("モルック"));
    }

    #[test]
    fn 蓄力中は照準候補と速度を表示する() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Frame { time: 500.0 });
        let html = dioxus_ssr::render_element(render(&state));
        assert!(html.contains("class=\"aimed\""));
        assert!(html.contains("5.0 m/s"));
    }

    #[test]
    fn 勝利と不戦勝と勝者なしの終了を表示する() {
        for (teams, expected) in [
            (vec![team("赤", &[10; 5]), team("青", &[])], "赤の勝利"),
            (vec![team("赤", &[0; 3]), team("青", &[])], "青の勝利"),
            (vec![team("赤", &[0; 3])], "勝者なし"),
            (vec![], "勝者なし"),
        ] {
            let state = Controller::new(teams);
            let html = dioxus_ssr::render_element(render(&state));
            assert!(html.contains(expected), "{html}");
            assert!(html.contains("data-phase=\"finished\""));
        }
    }
}

mod fixed_field {
    use super::*;

    #[test]
    fn 高速で投げても投擲中と停止後の表示範囲は変わらない() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        let initial = dioxus_ssr::render_element(render(&state));
        let bounds = initial
            .split("viewBox=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(bounds, "-0.3 -1.35 0.6 1.5");
        state.handle(DomEvent::Press(pointer([0.2, 0.2], 0.0)));
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 1000.0)));
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 1000.0 + f64::from(tick) * 20.0,
            });
            let html = dioxus_ssr::render_element(render(&state));
            assert!(
                html.contains(&format!("viewBox=\"{bounds}\"")),
                "表示範囲が変わりました"
            );
            if state.game().phase != GamePhase::Simulating {
                break;
            }
        }
        assert_eq!(state.game().phase, GamePhase::Ready);
        assert_eq!(state.game().teams[0].score_board().history().len(), 1);
        assert!(state.game().teams[1].turn());
    }
}
