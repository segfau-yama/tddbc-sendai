use super::*;
use crate::Name;

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

fn result(state: &mut AppState, numbers: &[i32]) {
    let fallen: Vec<_> = numbers
        .iter()
        .map(|number| {
            state
                .field
                .skittles()
                .iter()
                .find(|skittle| skittle.number() == *number)
                .unwrap()
                .with_state(SkittleState::Fall)
        })
        .collect();
    state.phase = Phase::Simulating;
    state.finish_throw(&fallen);
}

mod input_gating {
    use super::*;

    #[test]
    fn 蓄力中と物理計算中は新しい入力を開始しない() {
        for phase in [Phase::Charging, Phase::Simulating] {
            let mut state = AppState::new(vec![team("赤", &[])]);
            state.phase = phase;
            state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
            assert_eq!(state.input.pointer_id(), None);
        }
    }
}

mod composition {
    use super::*;

    #[test]
    fn 描画は物理のhitを呼ばずモデルに反映された転倒状態を使う() {
        let mut state = AppState::new(vec![team("赤", &[])]);
        let mut pins = state.field.skittles().to_vec();
        pins[0] = pins[0].with_state(SkittleState::Fall);
        state.field = GuiField::new(*state.field.as_ref(), pins);
        let html = dioxus_ssr::render_element(render(&state));
        assert_eq!(html.matches("data-state=\"fall\"").count(), 1);
        assert_eq!(html, dioxus_ssr::render_element(render(&state)));
    }

    #[test]
    fn 最初の手番と十二本と得点表を描画する() {
        let state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        assert!(state.teams[0].turn());
        assert!(!state.teams[1].turn());
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
            if state.phase != Phase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert!(state.teams[1].turn());
    }
}

mod initialization {
    use super::*;

    #[test]
    fn チームを保持して共通フィールドと棒を用意し待機から始める() {
        let teams = vec![
            Team::new(Name::new("赤")).with_turn(true),
            Team::new(Name::new("青")),
        ];
        let ids: Vec<_> = teams.iter().map(Team::id).collect();
        let state = AppState::new(teams);

        assert_eq!(state.teams.iter().map(Team::id).collect::<Vec<_>>(), ids);
        assert!(state.teams[0].turn());
        assert!(!state.teams[1].turn());
        assert_eq!(state.field.skittles().len(), 12);
        assert_eq!(state.stick.target(), None);
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.phase, Phase::Ready);
    }

    #[test]
    fn 参加者がいないときは終了して入力を受け付けない() {
        let mut state = AppState::new(vec![]);
        assert!(state.is_finished());
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert!(state.is_finished());
        assert_eq!(state.input.pointer_id(), None);
    }
}

mod input {
    use super::*;

    #[test]
    fn 早期転倒を保持して連鎖した二本を一投で採点し次の照準前に立て直す() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.field.as_ref().id();
        let pins = vec![Skittle::new(5), Skittle::new(8)];
        let ids: Vec<_> = pins.iter().map(Skittle::id).collect();
        state.rules = GuiMolkky::new(
            &state.stick,
            GuiField::THROW_POSITION,
            &[(ids[0], [0.02, 0.7]), (ids[1], [0.06, 0.85])],
        );
        state.field = GuiField::new(*state.field.as_ref(), pins);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        let mut fallen_ids = std::collections::HashSet::new();
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 600.0 + f64::from(tick) * 20.0,
            });
            if state.phase != Phase::Simulating {
                break;
            }
            for pin in state.field.skittles() {
                if fallen_ids.contains(&pin.id()) {
                    assert_eq!(pin.state(), SkittleState::Fall);
                }
                if pin.state() == SkittleState::Fall {
                    fallen_ids.insert(pin.id());
                }
            }
        }
        assert_eq!(fallen_ids.len(), 2);
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(
            state.teams[0]
                .score_board()
                .history()
                .iter()
                .map(|p| p.value())
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(state.teams[1].turn());
        assert_eq!(state.field.as_ref().id(), field_id);
        assert_eq!(
            state
                .field
                .skittles()
                .iter()
                .map(Skittle::id)
                .collect::<Vec<_>>(),
            ids
        );
        assert!(
            state
                .field
                .skittles()
                .iter()
                .all(|p| p.state() == SkittleState::Stand)
        );
        let positions: Vec<_> = ids
            .iter()
            .map(|&id| state.rules.pose(id).unwrap())
            .collect();
        state.handle(DomEvent::Frame { time: 50000.0 });
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert_eq!(
            ids.iter()
                .map(|&id| state.rules.pose(id).unwrap())
                .collect::<Vec<_>>(),
            positions
        );
        let aimed = GuiStick::new(*state.stick.as_ref(), Some(positions[0].0), 6.0);
        assert!(
            state
                .rules
                .aim(&aimed, state.field.skittles())
                .iter()
                .any(|p| p.id() == ids[0])
        );
    }

    #[test]
    fn 所有pointerの取消で照準と速度と入力状態を一緒に解除する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let stick_id = state.stick.stick().id();
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.stick.speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 1 });
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(state.input.pointer_id(), None);
        assert_eq!(state.stick.target(), None);
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.stick.stick().id(), stick_id);
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        assert!(state.teams[0].score_board().history().is_empty());
        assert!(state.teams[0].turn());
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
        assert_eq!(state.stick.target(), Some([0.0, 1.0]));
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.phase, Phase::Charging);
    }

    #[test]
    fn 蓄力と取消を接続し無効な方向では手番を進めない() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.phase, Phase::Charging);

        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.stick.speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 2 });
        assert_eq!(state.phase, Phase::Charging);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(state.stick.target(), None);
        state.handle(DomEvent::Press(pointer([0.0, 0.0], 1000.0)));
        state.handle(DomEvent::Release(pointer([0.0, 0.0], 1500.0)));
        assert_eq!(state.phase, Phase::Ready);
        assert!(state.teams[0].turn());
        assert!(state.teams[0].score_board().history().is_empty());
    }

    #[test]
    fn 実際の物理停止後だけ一度採点し交代する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.field.as_ref().id();
        let stick_id = state.stick.stick().id();
        state.handle(DomEvent::Press(pointer([0.2, 0.2], 0.0)));
        // 壁で反射してもスキットルへ届かない強さでミスを作る。
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 100.0)));
        assert_eq!(state.phase, Phase::Simulating);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase, Phase::Simulating);
        assert!(state.teams[0].score_board().history().is_empty());
        // タブ復帰の大きな時間差でも、1フレームで採点まで進めない。
        state.handle(DomEvent::Frame { time: 30000.0 });
        assert_eq!(state.phase, Phase::Simulating);
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 30000.0 + f64::from(tick) * 20.0,
            });
            if state.phase != Phase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert_eq!(state.teams[0].score_board().total(), 0);
        assert!(state.teams[1].turn());
        state.handle(DomEvent::Frame { time: 100000.0 });
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert_eq!(state.field.as_ref().id(), field_id);
        assert_eq!(state.stick.stick().id(), stick_id);
    }
}

mod scoring {
    use super::*;

    #[test]
    fn 複数本の番号を合計せずcoreで採点する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        result(&mut state, &[5, 8]);
        assert_eq!(state.teams[0].score_board().total(), 2);
        state.finish_throw(&[]);
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert!(state.teams[1].score_board().history().is_empty());
    }

    #[test]
    fn 超過は二十五点へ戻して次チームへ進む() {
        let mut state = AppState::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[2]);
        assert_eq!(state.teams[0].score_board().total(), 25);
        assert_eq!(state.phase, Phase::Ready);
        assert!(state.teams[1].turn());
    }

    #[test]
    fn 五十点なら終了して以降の入力を無効にする() {
        let mut state = AppState::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[1]);
        assert!(state.is_finished());
        assert!(state.teams.iter().all(|t| !t.turn()));
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.input.pointer_id(), None);
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 500.0)));
        assert_eq!(state.teams[0].score_board().history().len(), 6);
    }

    #[test]
    fn 三連続ミスで失格し最後の生存者が履歴を増やさず勝つ() {
        let mut state = AppState::new(vec![team("赤", &[0, 0]), team("青", &[])]);
        result(&mut state, &[]);
        assert!(state.is_finished());
        assert_eq!(
            state.teams[0].score_board().state(),
            crate::ScoreState::Disqualified
        );
        assert_eq!(state.teams[1].score_board().total(), 50);
        assert!(state.teams[1].score_board().history().is_empty());
    }

    #[test]
    fn 単独の失格は勝者なしで終了する() {
        let mut state = AppState::new(vec![team("赤", &[0, 0])]);
        result(&mut state, &[]);
        assert!(state.is_finished());
    }
}
