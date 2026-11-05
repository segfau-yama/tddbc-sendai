use super::super::stick::tests::support::*;
use super::super::stick::{DomEvent, Phase};
use crate::SkittleState;
use crate::gui_2d::{models::GuiSkittle, services::Game};

mod composition {
    use super::*;

    #[test]
    fn 描画は物理のhitを呼ばずモデルに反映された転倒状態を使う() {
        let mut state = TestState::new(vec![team("赤", &[])]);
        let mut skittles = state.game.skittles().to_vec();
        skittles[0] = GuiSkittle::new(
            skittles[0].as_ref().with_state(SkittleState::Fall),
            skittles[0].position(),
        );
        state.game = Game::new(
            state.game.teams().to_vec(),
            state.game.field().clone(),
            *state.game.stick(),
            skittles,
        );
        let html = render_layout(&state);
        assert_eq!(html.matches("data-state=\"fall\"").count(), 1);
        assert_eq!(html, render_layout(&state));
    }

    #[test]
    fn 最初の手番と十二本と得点表を描画する() {
        let state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
        assert!(state.game.teams()[0].turn());
        assert!(!state.game.teams()[1].turn());
        let html = render_layout(&state);
        assert_eq!(html.matches("data-state=\"stand\"").count(), 12);
        assert!(html.contains("data-phase=\"ready\""));
        assert!(html.contains("得点表"));
        assert!(html.contains("<meter"));
        assert!(html.contains("モルック"));
    }

    #[test]
    fn 蓄力中は照準候補と速度を表示する() {
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Frame { time: 500.0 });
        let html = render_layout(&state);
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
            let state = TestState::new(teams);
            let html = render_layout(&state);
            assert!(html.contains(expected), "{html}");
            assert!(html.contains("data-phase=\"finished\""));
        }
    }
}

mod fixed_field {
    use super::*;

    #[test]
    fn 高速で投げても投擲中と停止後の表示範囲は変わらない() {
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
        let initial = render_layout(&state);
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
            let html = render_layout(&state);
            assert!(
                html.contains(&format!("viewBox=\"{bounds}\"")),
                "表示範囲が変わりました"
            );
            if state.phase() != Phase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(state.game.teams()[0].score_board().history().len(), 1);
        assert!(state.game.teams()[1].turn());
    }
}
