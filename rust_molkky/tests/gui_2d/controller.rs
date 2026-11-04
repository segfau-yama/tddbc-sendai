use super::*;
use crate::core::{Name, SkittleState};

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

fn result(state: &mut Controller, numbers: &[i32]) {
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
    state.phase = InputPhase::Simulating;
    state.finish_throw(&fallen);
}

mod coordinates {
    use super::*;

    #[test]
    fn 描画領域の位置と拡縮と上下を反映する() {
        let rect = [100.0, 50.0, 400.0, 800.0];
        let bounds = [-2.0, 0.0, 4.0, 8.0];
        for (screen, world) in [
            ([100.0, 50.0], [-2.0, 8.0]),
            ([300.0, 450.0], [0.0, 4.0]),
            ([500.0, 850.0], [2.0, 0.0]),
        ] {
            assert_eq!(screen_to_world(screen, rect, bounds), Some(world));
        }
    }

    #[test]
    fn アスペクト比を保った縦横の余白は狙えない() {
        let wide = [0.0, 0.0, 800.0, 400.0];
        let tall = [0.0, 0.0, 400.0, 800.0];
        assert_eq!(
            screen_to_world([299.0, 0.0], wide, [-2.0, 0.0, 4.0, 8.0]),
            None
        );
        assert_eq!(
            screen_to_world([300.0, 0.0], wide, [-2.0, 0.0, 4.0, 8.0]),
            Some([-2.0, 8.0])
        );
        assert_eq!(
            screen_to_world([0.0, 299.0], tall, [-4.0, 0.0, 8.0, 4.0]),
            None
        );
        assert_eq!(
            screen_to_world([0.0, 300.0], tall, [-4.0, 0.0, 8.0, 4.0]),
            Some([-4.0, 4.0])
        );
    }

    #[test]
    fn 不正な寸法と非有限座標と領域外を拒否する() {
        let rect = [0.0, 0.0, 100.0, 100.0];
        let bounds = [0.0, 0.0, 1.0, 1.0];
        for cursor in [
            [f64::NAN, 0.0],
            [f64::INFINITY, 0.0],
            [-1.0, 0.0],
            [101.0, 50.0],
        ] {
            assert_eq!(screen_to_world(cursor, rect, bounds), None);
        }
        for invalid in [
            [0.0, 0.0, 0.0, 100.0],
            [0.0, 0.0, 100.0, -1.0],
            [0.0, 0.0, f64::INFINITY, 100.0],
        ] {
            assert_eq!(screen_to_world([50.0, 50.0], invalid, bounds), None);
        }
        for invalid in [
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, -1.0],
            [f32::NAN, 0.0, 1.0, 1.0],
        ] {
            assert_eq!(screen_to_world([50.0, 50.0], rect, invalid), None);
        }
    }
}

mod charging {
    use super::*;

    #[test]
    fn 待機中の左ボタンだけ開始し再押下で上書きしない() {
        let mut input = Controller::new(vec![team("赤", &[])]);
        let now = Duration::from_secs(10);
        for phase in [InputPhase::Charging, InputPhase::Simulating] {
            input.phase = phase;
            assert!(!input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        }
        input.phase = InputPhase::Ready;
        for button in [
            None,
            Some(MouseButton::Secondary),
            Some(MouseButton::Auxiliary),
        ] {
            assert!(!input.press(1, button, Some([0.0, 3.0]), now));
        }
        for target in [None, Some([f32::NAN, 3.0])] {
            assert!(!input.press(1, Some(MouseButton::Primary), target, now));
        }
        assert_eq!(input.pointer_id(), None);
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        assert!(!input.press(2, Some(MouseButton::Primary), Some([1.0, 3.0]), now));
        assert!(!input.press(1, Some(MouseButton::Primary), Some([1.0, 3.0]), now));
        assert_eq!(input.pointer_id(), Some(1));
        assert_eq!(input.preview(now).unwrap().target(), Some([0.0, 3.0]));
    }

    #[test]
    fn 経過時間で速度が上下し開始前の時刻は零とする() {
        let mut input = Controller::new(vec![team("赤", &[])]);
        let stick = input.stick;
        let start = Duration::from_secs(10);
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), start));
        for (millis, speed) in [
            (0, 0.0),
            (500, 5.0),
            (1000, 10.0),
            (1500, 5.0),
            (2000, 0.0),
            (2500, 5.0),
        ] {
            let preview = input
                .preview(start + Duration::from_millis(millis))
                .unwrap();
            assert_eq!(preview.speed(), speed);
            assert_eq!(preview.stick().id(), stick.stick().id());
        }
        assert_eq!(input.preview(Duration::ZERO).unwrap().speed(), 0.0);
        assert_eq!(stick.target(), None);
    }

    #[test]
    fn 同じpointerの有効な移動だけ照準に反映する() {
        let mut input = Controller::new(vec![team("赤", &[])]);
        let now = Duration::ZERO;
        input.move_to(1, Some([1.0, 1.0]));
        assert!(input.preview(now).is_none());
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.move_to(2, Some([1.0, 2.0]));
        assert_eq!(input.preview(now).unwrap().target(), Some([0.0, 3.0]));
        input.move_to(1, Some([1.0, 2.0]));
        input.move_to(1, None);
        input.move_to(1, Some([f32::INFINITY, 2.0]));
        assert_eq!(input.preview(now).unwrap().target(), Some([1.0, 2.0]));
    }

    #[test]
    fn 領域外でも所有pointerの左解放で表示と同じ初速を一度だけ返す() {
        let mut input = Controller::new(vec![team("赤", &[])]);
        let stick = input.stick;
        assert!(input.press(
            1,
            Some(MouseButton::Primary),
            Some([0.0, 3.0]),
            Duration::ZERO
        ));
        let now = Duration::from_millis(500);
        assert!(input.release(2, Some(MouseButton::Primary), now).is_none());
        assert!(
            input
                .release(1, Some(MouseButton::Secondary), now)
                .is_none()
        );
        let preview = input.preview(now).unwrap();
        input.move_to(1, None);
        let thrown = input.release(1, Some(MouseButton::Primary), now).unwrap();
        assert_eq!(thrown.target(), preview.target());
        assert_eq!(thrown.speed(), preview.speed());
        assert_eq!(thrown.stick().id(), stick.stick().id());
        assert_eq!(input.pointer_id(), None);
        assert!(input.preview(now).is_none());
        assert!(input.release(1, Some(MouseButton::Primary), now).is_none());
    }
}

mod cancellation {
    use super::*;

    #[test]
    fn 該当pointerの取消とfocus喪失では投擲しない() {
        let mut input = Controller::new(vec![team("赤", &[])]);
        let now = Duration::ZERO;
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.cancel_pointer(2);
        assert_eq!(input.pointer_id(), Some(1));
        input.cancel_pointer(1);
        assert!(input.release(1, Some(MouseButton::Primary), now).is_none());
        assert!(input.press(2, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.cancel();
        assert!(input.preview(now).is_none());
        assert!(input.release(2, Some(MouseButton::Primary), now).is_none());
        let later = Duration::from_secs(10);
        assert!(input.press(3, Some(MouseButton::Primary), Some([0.0, 3.0]), later));
        assert_eq!(input.preview(later).unwrap().speed(), 0.0);
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
        let state = Controller::new(teams);

        assert_eq!(state.teams.iter().map(Team::id).collect::<Vec<_>>(), ids);
        assert!(state.teams[0].turn());
        assert!(!state.teams[1].turn());
        assert_eq!(state.field.skittles().len(), 12);
        assert_eq!(state.stick.target(), None);
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.phase, InputPhase::Ready);
    }

    #[test]
    fn 参加者がいないときは終了して入力を受け付けない() {
        let mut state = Controller::new(vec![]);
        assert!(state.is_finished());
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert!(state.is_finished());
        assert_eq!(state.pointer_id(), None);
    }
}

mod input {
    use super::*;

    #[test]
    fn 所有pointerの取消で照準と速度と入力状態を一緒に解除する() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        let stick_id = state.stick.stick().id();
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.stick.speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 1 });
        assert_eq!(state.phase, InputPhase::Ready);
        assert_eq!(state.pointer_id(), None);
        assert_eq!(state.stick.target(), None);
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.stick.stick().id(), stick_id);
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 600.0)));
        assert!(state.teams[0].score_board().history().is_empty());
        assert!(state.teams[0].turn());
    }

    #[test]
    fn 不正な時刻のイベントで照準や入力状態を変更しない() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        state.handle(DomEvent::Move(pointer([0.2, 0.2], f64::NAN)));
        state.handle(DomEvent::Frame {
            time: f64::INFINITY,
        });
        state.handle(DomEvent::Release(pointer([0.0, 1.0], -1.0)));
        assert_eq!(state.stick.target(), Some([0.0, 1.0]));
        assert_eq!(state.stick.speed(), 0.0);
        assert_eq!(state.phase, InputPhase::Charging);
    }

    #[test]
    fn 蓄力と取消を接続し無効な方向では手番を進めない() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.phase, InputPhase::Charging);

        state.handle(DomEvent::Frame { time: 500.0 });
        assert_eq!(state.stick.speed(), 5.0);
        state.handle(DomEvent::Cancel { pointer_id: 2 });
        assert_eq!(state.phase, InputPhase::Charging);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase, InputPhase::Ready);
        assert_eq!(state.stick.target(), None);
        state.handle(DomEvent::Press(pointer([0.0, 0.0], 1000.0)));
        state.handle(DomEvent::Release(pointer([0.0, 0.0], 1500.0)));
        assert_eq!(state.phase, InputPhase::Ready);
        assert!(state.teams[0].turn());
        assert!(state.teams[0].score_board().history().is_empty());
    }

    #[test]
    fn 実際の物理停止後だけ一度採点し交代する() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = state.field.field().id();
        let stick_id = state.stick.stick().id();
        state.handle(DomEvent::Press(pointer([0.2, 0.2], 0.0)));
        // 壁で反射してもスキットルへ届かない強さでミスを作る。
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 100.0)));
        assert_eq!(state.phase, InputPhase::Simulating);
        state.handle(DomEvent::Blur);
        assert_eq!(state.phase, InputPhase::Simulating);
        assert!(state.teams[0].score_board().history().is_empty());
        // タブ復帰の大きな時間差でも、1フレームで採点まで進めない。
        state.handle(DomEvent::Frame { time: 30000.0 });
        assert_eq!(state.phase, InputPhase::Simulating);
        for tick in 1..1200 {
            state.handle(DomEvent::Frame {
                time: 30000.0 + f64::from(tick) * 20.0,
            });
            if state.phase != InputPhase::Simulating {
                break;
            }
        }
        assert_eq!(state.phase, InputPhase::Ready);
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert_eq!(state.teams[0].score_board().total(), 0);
        assert!(state.teams[1].turn());
        state.handle(DomEvent::Frame { time: 100000.0 });
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert_eq!(state.field.field().id(), field_id);
        assert_eq!(state.stick.stick().id(), stick_id);
    }
}

mod scoring {
    use super::*;

    #[test]
    fn 複数本の番号を合計せずcoreで採点する() {
        let mut state = Controller::new(vec![team("赤", &[]), team("青", &[])]);
        result(&mut state, &[5, 8]);
        assert_eq!(state.teams[0].score_board().total(), 2);
        state.finish_throw(&[]);
        assert_eq!(state.teams[0].score_board().history().len(), 1);
        assert!(state.teams[1].score_board().history().is_empty());
    }

    #[test]
    fn 超過は二十五点へ戻して次チームへ進む() {
        let mut state = Controller::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[2]);
        assert_eq!(state.teams[0].score_board().total(), 25);
        assert_eq!(state.phase, InputPhase::Ready);
        assert!(state.teams[1].turn());
    }

    #[test]
    fn 五十点なら終了して以降の入力を無効にする() {
        let mut state = Controller::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        result(&mut state, &[1]);
        assert!(state.is_finished());
        assert!(state.teams.iter().all(|t| !t.turn()));
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        assert_eq!(state.pointer_id(), None);
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 500.0)));
        assert_eq!(state.teams[0].score_board().history().len(), 6);
    }

    #[test]
    fn 三連続ミスで失格し最後の生存者が履歴を増やさず勝つ() {
        let mut state = Controller::new(vec![team("赤", &[0, 0]), team("青", &[])]);
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
        let mut state = Controller::new(vec![team("赤", &[0, 0])]);
        result(&mut state, &[]);
        assert!(state.is_finished());
    }
}
