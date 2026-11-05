use super::*;
use crate::gui_2d::{
    models::{GuiField, GuiSkittle},
    services::field::{create_field, create_world},
};
use crate::{Molkky, Skittle, SkittleState};
use crate::{Name, Stick as CoreStick, StickState, Team, Throw};
use support::*;

mod geometry {
    use super::*;
    #[test]
    fn モデルの識別子とラジアン角と寸法で棒を描く() {
        let model = GuiStick::throw(GuiStick::new(CoreStick::new(), Some([1.0, 0.0]), 6.0));
        let html = render(model);
        assert!(html.contains(&format!("stick-{}", model.as_ref().id())));
        assert!(html.contains("translate(0 0) rotate(-90)"), "{html}");
        assert!(html.contains(&format!("width=\"{}\"", 2.0 * GuiStick::RADIUS)));
        assert!(html.contains(&format!("height=\"{}\"", GuiStick::LENGTH)));
    }
}

mod next_throw {
    use super::*;
    #[test]
    fn 初期化した次投のモデルは物理更新前も投擲位置で表示される() {
        let mut model = GuiStick::throw(GuiStick::new(CoreStick::new(), Some([0.0, 1.0]), 6.0));
        let mut skittles = vec![];
        let mut rules = create_world(&model, &skittles);
        for _ in 0..1200 {
            (skittles, model) = rules.update(&skittles, model);
            if model.as_ref().state() == StickState::Stop {
                break;
            }
        }
        assert_eq!(model.as_ref().state(), StickState::Stop);
        assert_ne!(model.position(), GuiField::THROW_POSITION);
        let held = GuiStick::new(*model.as_ref(), Some([0.0, 1.0]), 6.0);
        let html = render(held);
        assert!(html.contains("translate(0 0) rotate(0)"), "{html}");
        assert!(html.contains("data-stick-state=\"held\""));
        let thrown = GuiStick::throw(held);
        assert_eq!(thrown.position(), GuiField::THROW_POSITION);
        let thrown_html = render(thrown);
        assert!(thrown_html.contains("translate(0 0) rotate(0)"));
        assert!(thrown_html.contains("data-stick-state=\"move\""));
        assert_eq!(
            thrown_html
                .split("</g>")
                .next()
                .unwrap()
                .replace("data-stick-state=\"move\"", "data-stick-state=\"held\""),
            html.split("</g>").next().unwrap(),
        );
        assert!(html.contains("<line"));
        assert!(!thrown_html.contains("<line"));
        let (_, moved) = rules.update(&skittles, thrown);
        assert!(!render(moved).contains("translate(0 0) rotate(0)"));
    }
}

fn render(model: GuiStick) -> String {
    fn fixture(model: GuiStick) -> Element {
        let game = use_signal(|| {
            crate::gui_2d::services::Game::new(
                vec![crate::Team::new(crate::Name::new("赤"))],
                GuiField::new(crate::Field::new()),
                model,
                vec![],
            )
        });
        rsx! { svg { Stick { game } } }
    }
    let mut dom = VirtualDom::new_with_props(fixture, model);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
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
        let mut input = StickInput::default();
        let now = Duration::from_secs(10);
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
        assert_eq!(input.preview(now).unwrap().0, [0.0, 3.0]);
    }

    #[test]
    fn 経過時間で速度が上下し開始前の時刻は零とする() {
        let mut input = StickInput::default();
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
            assert_eq!(preview.1, speed);
        }
        assert_eq!(input.preview(Duration::ZERO).unwrap().1, 0.0);
    }

    #[test]
    fn 同じpointerの有効な移動だけ照準に反映する() {
        let mut input = StickInput::default();
        let now = Duration::ZERO;
        input.move_to(1, Some([1.0, 1.0]));
        assert!(input.preview(now).is_none());
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.move_to(2, Some([1.0, 2.0]));
        assert_eq!(input.preview(now).unwrap().0, [0.0, 3.0]);
        input.move_to(1, Some([1.0, 2.0]));
        input.move_to(1, None);
        input.move_to(1, Some([f32::INFINITY, 2.0]));
        assert_eq!(input.preview(now).unwrap().0, [1.0, 2.0]);
    }

    #[test]
    fn 領域外でも所有pointerの左解放で表示と同じ初速を一度だけ返す() {
        let mut input = StickInput::default();
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
        assert_eq!(thrown.0, preview.0);
        assert_eq!(thrown.1, preview.1);
        assert_eq!(input.pointer_id(), None);
        assert!(input.preview(now).is_none());
        assert!(input.release(1, Some(MouseButton::Primary), now).is_none());
    }
}

mod cancellation {
    use super::*;

    #[test]
    fn 該当pointerの取消とfocus喪失では投擲しない() {
        let mut input = StickInput::default();
        let now = Duration::ZERO;
        assert!(input.press(1, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.cancel(Some(2));
        assert_eq!(input.pointer_id(), Some(1));
        input.cancel(Some(1));
        assert!(input.release(1, Some(MouseButton::Primary), now).is_none());
        assert!(input.press(2, Some(MouseButton::Primary), Some([0.0, 3.0]), now));
        input.cancel(None);
        assert!(input.preview(now).is_none());
        assert!(input.release(2, Some(MouseButton::Primary), now).is_none());
        let later = Duration::from_secs(10);
        assert!(input.press(3, Some(MouseButton::Primary), Some([0.0, 3.0]), later));
        assert_eq!(input.preview(later).unwrap().1, 0.0);
    }
}

mod input_gating {
    use super::*;

    #[test]
    fn 蓄力中と物理計算中は新しい入力を開始しない() {
        for simulating in [false, true] {
            let mut state = TestState::new(vec![team("赤", &[])]);
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

mod input {
    use super::*;

    #[test]
    fn 解放後の固定刻み未満のフレームでは採点せず次の刻みで移動を始める() {
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
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

pub(in crate::gui_2d::views) mod support {
    use super::*;

    pub struct TestState {
        pub game: Game,
        pub input: StickInput,
    }
    impl TestState {
        pub fn new(teams: Vec<Team>) -> Self {
            let (field, stick, skittles) = create_field();
            Self {
                game: Game::new(teams, field, stick, skittles),
                input: StickInput::default(),
            }
        }
        pub fn handle(&mut self, event: DomEvent) {
            self.input.handle(&mut self.game, event);
        }
        pub fn phase(&self) -> Phase {
            phase(&self.game)
        }
    }

    pub fn team(name: &str, points: &[i32]) -> Team {
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

    pub fn pointer(target: [f64; 2], time: f64) -> PointerSample {
        PointerSample {
            pointer_id: 1,
            button: 0,
            cursor: [(target[0] + 1.0) * 100.0, (3.0 - target[1]) * 100.0],
            rect: [0.0, 0.0, 200.0, 400.0],
            bounds: [-1.0, -1.0, 2.0, 4.0],
            time,
        }
    }

    pub fn render_layout(state: &TestState) -> String {
        type Snapshot = (Vec<Team>, GuiField, GuiStick, Vec<GuiSkittle>);
        fn fixture((teams, field, stick, skittles): Snapshot) -> Element {
            let game = use_signal(|| Game::new(teams, field, stick, skittles));
            super::super::super::app::render(game)
        }
        let snapshot = (
            state.game.teams().to_vec(),
            state.game.field().clone(),
            *state.game.stick(),
            state.game.skittles().to_vec(),
        );
        let mut dom = VirtualDom::new_with_props(fixture, snapshot);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod stick_states {
    use super::*;

    #[test]
    fn 押下と蓄力は保持と零速度を保ち解放した表示速度で投げて減速する() {
        let mut state = TestState::new(vec![team("赤", &[]), team("青", &[])]);
        let id = state.game.stick().as_ref().id();
        let origin = state.game.stick().position();
        state.handle(DomEvent::Press(pointer([0.0, 1.0], 0.0)));
        for (time, speed) in [(500.0, 5.0), (1000.0, 10.0), (1500.0, 5.0)] {
            state.handle(DomEvent::Frame { time });
            let held = state.game.stick();
            assert_eq!(held.as_ref().state(), StickState::Held);
            assert_eq!(held.speed(), speed);
            assert_eq!(held.velocity(), [0.0; 2]);
            assert_eq!(held.position(), origin);
            assert_eq!(held.as_ref().id(), id);
            assert!(state.game.teams()[0].score_board().history().is_empty());
        }
        state.handle(DomEvent::Release(pointer([0.0, 1.0], 1500.0)));
        assert_eq!(state.game.stick().as_ref().state(), StickState::Move);
        assert_eq!(state.game.stick().speed(), 5.0);
        assert_eq!(state.game.stick().velocity(), [0.0, 5.0]);
        assert_eq!(state.game.stick().position(), origin);
        state.handle(DomEvent::Frame { time: 1520.0 });
        let moving = state.game.stick();
        assert_eq!(moving.as_ref().state(), StickState::Move);
        assert!(moving.speed() > 0.0 && moving.speed() < 5.0);
        assert_ne!(moving.position(), origin);
        assert_eq!(moving.as_ref().id(), id);
        let speed = moving.speed();
        state.handle(DomEvent::Move(pointer([0.2, 0.2], 1600.0)));
        state.handle(DomEvent::Release(pointer([0.2, 0.2], 1600.0)));
        assert_eq!(state.game.stick().speed(), speed);
        assert_eq!(state.game.stick().target(), Some([0.0, 1.0]));
    }
}
