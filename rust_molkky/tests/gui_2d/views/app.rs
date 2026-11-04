use super::*;
use crate::Name;
use crate::gui_2d::models::{GuiSkittle, GuiStick};
use crate::gui_2d::services::field::create_world;
use crate::{Molkky, Skittle, SkittleState};

fn team(name: &str, points: &[i32]) -> Team {
    let stick = GuiStick::default();
    let rules = create_world(&stick, &[]);
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

mod input_gating {
    use super::*;

    #[test]
    fn 蓄力中と物理計算中は新しい入力を開始しない() {
        for simulating in [false, true] {
            let mut state = AppState::new(vec![team("赤", &[])]);
            state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
            if simulating {
                state.handle(DomEvent::Release(pointer([0.0, 1.0], 500.0)));
            }
            let owner = state.input.pointer_id();
            let target = state.game.stick().target();
            state.handle(DomEvent::Press(pointer([0.2, 0.2], 600.0)));
            assert_eq!(state.input.pointer_id(), owner);
            assert_eq!(state.game.stick().target(), target);
        }
    }
}

mod composition {
    use super::*;

    #[test]
    fn 描画は物理のhitを呼ばずモデルに反映された転倒状態を使う() {
        let mut state = AppState::new(vec![team("赤", &[])]);
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
        let html = dioxus_ssr::render_element(render(&state));
        assert_eq!(html.matches("data-state=\"fall\"").count(), 1);
        assert_eq!(html, dioxus_ssr::render_element(render(&state)));
    }

    #[test]
    fn 最初の手番と十二本と得点表を描画する() {
        let state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        assert!(state.game.teams()[0].turn());
        assert!(!state.game.teams()[1].turn());
        let html = dioxus_ssr::render_element(render(&state));
        assert_eq!(html.matches("data-state=\"stand\"").count(), 12);
        assert!(html.contains("data-phase=\"ready\""));
        assert!(html.contains("得点表"));
        assert!(html.contains("<meter"));
        assert!(html.contains("モルック"));
    }

    #[test]
    fn 蓄力中は照準候補と速度を表示する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
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
            let state = AppState::new(teams);
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
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
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
            if state.phase() != Phase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(state.game.teams()[0].score_board().history().len(), 1);
        assert!(state.game.teams()[1].turn());
    }
}

mod input {
    use super::*;

    #[test]
    fn 解放後の固定刻み未満のフレームでは採点せず次の刻みで移動を始める() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        let origin = state.game.stick().position();
        state.handle(DomEvent::Frame { time: 601.0 });
        assert_eq!(state.phase(), Phase::Simulating);
        assert!(state.game.teams()[0].score_board().history().is_empty());
        assert!(state.game.teams()[0].turn());
        assert_eq!(state.game.stick().position(), origin);
        state.handle(DomEvent::Frame { time: 620.0 });
        assert_ne!(state.game.stick().position(), origin);
        assert!(state.game.teams()[0].score_board().history().is_empty());
    }

    #[test]
    fn 早期の命中を追跡して連鎖した二本を一投で採点し次の照準前に立て直す() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.game.field().as_ref().id();
        let skittles = vec![
            GuiSkittle::new(Skittle::new(5), [0.02, 0.7]),
            GuiSkittle::new(Skittle::new(8), [0.06, 0.85]),
        ];
        let ids: Vec<_> = skittles.iter().map(|p| p.as_ref().id()).collect();
        state.game = Game::new(
            state.game.teams().to_vec(),
            state.game.field().clone(),
            *state.game.stick(),
            skittles,
        );
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        let mut hit_ids = std::collections::HashSet::new();
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 600.0 + f64::from(tick) * 20.0,
            });
            if state.phase() != Phase::Simulating {
                break;
            }
            for skittle in state.game.skittles() {
                if skittle.as_ref().state() == SkittleState::Hit {
                    hit_ids.insert(skittle.as_ref().id());
                }
            }
        }
        assert_eq!(hit_ids.len(), 2);
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(
            state.game.teams()[0]
                .score_board()
                .history()
                .iter()
                .map(|p| p.value())
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(state.game.teams()[1].turn());
        assert_eq!(state.game.field().as_ref().id(), field_id);
        assert_eq!(
            state
                .game
                .skittles()
                .iter()
                .map(|p| p.as_ref().id())
                .collect::<Vec<_>>(),
            ids
        );
        assert!(
            state
                .game
                .skittles()
                .iter()
                .all(|p| p.as_ref().state() == SkittleState::Stand)
        );
        let positions: Vec<_> = state
            .game
            .skittles()
            .iter()
            .map(|p| (p.position(), p.angle()))
            .collect();
        state.handle(DomEvent::Frame { time: 50000.0 });
        assert_eq!(state.game.teams()[0].score_board().history().len(), 1);
        assert_eq!(
            state
                .game
                .skittles()
                .iter()
                .map(|p| (p.position(), p.angle()))
                .collect::<Vec<_>>(),
            positions
        );
        state.game.aim(positions[0].0, 6.0);
        assert!(
            state
                .game
                .skittles()
                .iter()
                .any(|skittle| skittle.aimed() && skittle.as_ref().id() == ids[0])
        );
    }

    #[test]
    fn 所有pointerの取消で照準と速度と入力状態を一緒に解除する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let stick_id = state.game.stick().stick().id();
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.game.stick().speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 1 });
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(state.input.pointer_id(), None);
        assert_eq!(state.game.stick().target(), None);
        assert_eq!(state.game.stick().speed(), 0.0);
        assert_eq!(state.game.stick().stick().id(), stick_id);
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        assert!(state.game.teams()[0].score_board().history().is_empty());
        assert!(state.game.teams()[0].turn());
    }

    #[test]
    fn 不正な時刻のイベントで照準や入力状態を変更しない() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Move(pointer([0.2, 0.2], f64::NAN)));
        state.handle(DomEvent::Frame {
            time: f64::INFINITY,
        });
        state.handle(DomEvent::Release(pointer([0.0, 1.0], -1.0)));
        assert_eq!(state.game.stick().target(), Some([0.0, 1.0]));
        assert_eq!(state.game.stick().speed(), 0.0);
        assert_eq!(state.phase(), Phase::Charging);
    }

    #[test]
    fn 蓄力と取消を接続し無効な方向では手番を進めない() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.phase(), Phase::Charging);

        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.game.stick().speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 2 });
        assert_eq!(state.phase(), Phase::Charging);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(state.game.stick().target(), None);
        state.handle(DomEvent::Press(pointer([0.0, 0.0], 1000.0)));
        state.handle(DomEvent::Release(pointer([0.0, 0.0], 1500.0)));
        assert_eq!(state.phase(), Phase::Ready);
        assert!(state.game.teams()[0].turn());
        assert!(state.game.teams()[0].score_board().history().is_empty());
    }

    #[test]
    fn 実際の物理停止後だけ一度採点し交代する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.game.field().as_ref().id();
        let stick_id = state.game.stick().stick().id();
        state.handle(DomEvent::Press(pointer([0.2, 0.2], 0.0)));
        // 壁で反射してもスキットルへ届かない強さでミスを作る。
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 100.0)));
        assert_eq!(state.phase(), Phase::Simulating);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase(), Phase::Simulating);
        assert!(state.game.teams()[0].score_board().history().is_empty());
        // タブ復帰の大きな時間差でも、1フレームで採点まで進めない。
        state.handle(DomEvent::Frame { time: 30000.0 });
        assert_eq!(state.phase(), Phase::Simulating);
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 30000.0 + f64::from(tick) * 20.0,
            });
            if state.phase() != Phase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase(), Phase::Ready);
        assert_eq!(state.game.teams()[0].score_board().history().len(), 1);
        assert_eq!(state.game.teams()[0].score_board().total(), 0);
        assert!(state.game.teams()[1].turn());
        state.handle(DomEvent::Frame { time: 100000.0 });
        assert_eq!(state.game.teams()[0].score_board().history().len(), 1);
        assert_eq!(state.game.field().as_ref().id(), field_id);
        assert_eq!(state.game.stick().stick().id(), stick_id);
    }
}
