use super::*;
use crate::gui_2d::controller::PointerSample;

fn team(name: &str, points: &[i32]) -> Team {
    let stick = GuiStick::default();
    let rules = GuiMolkky::new(&stick, AppState::ORIGIN, &[]);
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
                .game
                .field
                .skittles()
                .iter()
                .find(|pin| pin.number() == *number)
                .unwrap()
                .with_state(SkittleState::Fall)
        })
        .collect();
    state.game.phase = GamePhase::Simulating;
    state.finish_throw(&fallen);
}

mod initialization {
    use super::*;

    #[test]
    fn 標準配置と最初の手番を作り描画部品を合成する() {
        let state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        assert!(state.game.teams[0].turn());
        assert!(!state.game.teams[1].turn());
        let pin = |number| {
            state
                .game
                .field
                .skittles()
                .iter()
                .find(|p| p.number() == number)
                .unwrap()
                .id()
        };
        let left = state.rules.pose(pin(1)).unwrap().0;
        let right = state.rules.pose(pin(2)).unwrap().0;
        assert!(left[0] < right[0]);
        assert!(state.rules.pose(pin(10)).unwrap().0[1] > left[1]);
        assert!(state.rules.pose(pin(9)).unwrap().0[1] > state.rules.pose(pin(12)).unwrap().0[1]);
        let html = dioxus_ssr::render_element(state.render());
        assert_eq!(html.matches("data-state=\"stand\"").count(), 12);
        assert!(html.contains("data-phase=\"ready\""));
        assert!(html.contains("得点表"));
        assert!(html.contains("<meter"));
        assert!(html.contains("モルック"));
    }

    #[test]
    fn 参加者がいないときは勝者なしで終了する() {
        let mut state = AppState::new(vec![]);
        assert_eq!(state.game.phase, GamePhase::Finished);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.game.phase, GamePhase::Finished);
        assert!(dioxus_ssr::render_element(state.render()).contains("勝者なし"));
    }
}

mod input {
    use super::*;

    #[test]
    fn 不正な時刻のイベントで照準や入力状態を変更しない() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Move(pointer([0.2, 0.2], f64::NAN)));
        state.handle(DomEvent::Frame {
            time: f64::INFINITY,
        });
        state.handle(DomEvent::Release(pointer([0.0, 1.0], -1.0)));
        assert_eq!(state.game.stick.target(), Some([0.0, 1.0]));
        assert_eq!(state.game.stick.speed(), 0.0);
        assert_eq!(state.game.phase, GamePhase::Charging);
    }

    #[test]
    fn 蓄力と取消を接続し無効な方向では手番を進めない() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.game.phase, GamePhase::Charging);
        assert!(dioxus_ssr::render_element(state.render()).contains("class=\"aimed\""));
        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.game.stick.speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 2 });
        assert_eq!(state.game.phase, GamePhase::Charging);
        state.handle(DomEvent::Blur);
        assert_eq!(state.game.phase, GamePhase::Ready);
        assert_eq!(state.game.stick.target(), None);
        state.handle(DomEvent::Press(pointer([0.0, 0.0], 1000.0)));
        state.handle(DomEvent::Release(pointer([0.0, 0.0], 1500.0)));
        assert_eq!(state.game.phase, GamePhase::Ready);
        assert!(state.game.teams[0].turn());
        assert!(state.game.teams[0].score_board().history().is_empty());
    }

    #[test]
    fn 実際の物理停止後だけ一度採点し交代する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.game.field.field().id();
        let stick_id = state.game.stick.stick().id();
        state.handle(DomEvent::Press(pointer([0.2, 0.2], 0.0)));
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 500.0)));
        assert_eq!(state.game.phase, GamePhase::Simulating);
        state.handle(DomEvent::Blur);
        assert_eq!(state.game.phase, GamePhase::Simulating);
        assert!(state.game.teams[0].score_board().history().is_empty());
        // タブ復帰の大きな時間差でも、1フレームで採点まで進めない。
        state.handle(DomEvent::Frame { time: 30000.0 });
        assert_eq!(state.game.phase, GamePhase::Simulating);
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 30000.0 + f64::from(tick) * 20.0,
            });
            if state.game.phase != GamePhase::Simulating {
                break;
            }
        }
        assert_eq!(state.game.phase, GamePhase::Ready);
        assert_eq!(state.game.teams[0].score_board().history().len(), 1);
        assert_eq!(state.game.teams[0].score_board().total(), 0);
        assert!(state.game.teams[1].turn());
        state.handle(DomEvent::Frame { time: 100000.0 });
        assert_eq!(state.game.teams[0].score_board().history().len(), 1);
        assert_eq!(state.game.field.field().id(), field_id);
        assert_eq!(state.game.stick.stick().id(), stick_id);
    }
}

mod scoring {
    use super::*;

    #[test]
    fn 複数本の番号を合計せずcoreで採点する() {
        let mut state = AppState::new(vec![team("赤", &[]), team("青", &[])]);
        result(&mut state, &[5, 8]);
        assert_eq!(state.game.teams[0].score_board().total(), 2);
        state.finish_throw(&[]);
        assert_eq!(state.game.teams[0].score_board().history().len(), 1);
        assert!(state.game.teams[1].score_board().history().is_empty());
    }

    #[test]
    fn 超過は二十五点へ戻して次チームへ進む() {
        let mut state = AppState::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[2]);
        assert_eq!(state.game.teams[0].score_board().total(), 25);
        assert_eq!(state.game.phase, GamePhase::Ready);
        assert!(state.game.teams[1].turn());
    }

    #[test]
    fn 五十点なら終了して以降の入力を無効にする() {
        let mut state = AppState::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[1]);
        assert_eq!(state.game.phase, GamePhase::Finished);
        assert!(state.game.teams.iter().all(|t| !t.turn()));
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 500.0)));
        assert_eq!(state.game.teams[0].score_board().history().len(), 6);
        assert!(dioxus_ssr::render_element(state.render()).contains("赤の勝利"));
    }

    #[test]
    fn 三連続ミスで失格し最後の生存者が履歴を増やさず勝つ() {
        let mut state = AppState::new(vec![team("赤", &[0, 0]), team("青", &[])]);
        result(&mut state, &[]);
        assert_eq!(state.game.phase, GamePhase::Finished);
        assert_eq!(
            state.game.teams[0].score_board().state(),
            crate::ScoreState::Disqualified
        );
        assert_eq!(state.game.teams[1].score_board().total(), 50);
        assert!(state.game.teams[1].score_board().history().is_empty());
        assert!(dioxus_ssr::render_element(state.render()).contains("青の勝利"));
    }

    #[test]
    fn 単独の失格は勝者なしで終了する() {
        let mut state = AppState::new(vec![team("赤", &[0, 0])]);
        result(&mut state, &[]);
        assert_eq!(state.game.phase, GamePhase::Finished);
        assert!(dioxus_ssr::render_element(state.render()).contains("勝者なし"));
    }
}
