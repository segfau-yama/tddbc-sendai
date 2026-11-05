use super::*;
use crate::gui_2d::services::{field::create_world, physics};
use crate::{Move, Stick as CoreStick};

fn held() -> GuiStick {
    GuiStick::default()
}

fn render(model: GuiStick) -> String {
    fn fixture(model: GuiStick) -> Element {
        let model = use_signal(|| model);
        let clock = use_signal(|| Duration::ZERO);
        let surface = use_signal(|| None::<Rc<MountedData>>);
        let enabled = use_signal(|| true);
        rsx! { svg { Stick { model, clock, surface, enabled } } }
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

mod geometry {
    use super::*;
    #[test]
    fn モデルの識別子とラジアン角と寸法で棒を描く() {
        let model = GuiStick::r#move(GuiStick::new(CoreStick::new(), Some([1.0, 0.0]), 6.0));
        let html = render(model);
        assert!(html.contains(&format!("stick-{}", model.as_ref().id())));
        assert!(html.contains("translate(0 0) rotate(-90)"), "{html}");
        assert!(html.contains(&format!("width=\"{}\"", 2.0 * GuiStick::RADIUS)));
        assert!(html.contains(&format!("height=\"{}\"", GuiStick::LENGTH)));
    }
    #[test]
    fn 次投の棒は最初の物理更新前も保持位置を描く() {
        let mut model = GuiStick::r#move(GuiStick::new(CoreStick::new(), Some([0.0, 1.0]), 6.0));
        let mut pins = vec![];
        let mut rules = create_world(&model, &pins);
        for _ in 0..1200 {
            (pins, model) = physics::step(&mut rules, &pins, model);
            if model.as_ref().state() == StickState::Stop {
                break;
            }
        }
        assert_eq!(model.as_ref().state(), StickState::Stop);
        assert_ne!(model.position(), GuiField::THROW_POSITION);
        let held = GuiStick::new(*model.as_ref(), Some([0.0, 1.0]), 6.0);
        let html = render(held);
        assert!(html.contains("translate(0 0) rotate(0)"));
        assert!(html.contains("data-stick-state=\"held\""));
        let moved = GuiStick::r#move(held);
        let next = render(moved);
        assert!(next.contains("data-stick-state=\"move\""));
        let marker = format!("id=\"stick-{}\"", held.as_ref().id());
        assert_eq!(
            next.split(&marker)
                .nth(1)
                .unwrap()
                .split("</g>")
                .next()
                .unwrap()
                .replace("data-stick-state=\"move\"", "data-stick-state=\"held\""),
            html.split(&marker)
                .nth(1)
                .unwrap()
                .split("</g>")
                .next()
                .unwrap(),
        );
        assert!(html.contains("<line") && !next.contains("<line"));
        let (_, advanced) = physics::step(&mut rules, &pins, moved);
        assert!(!render(advanced).contains("translate(0 0) rotate(0)"));
    }
}

mod charging {
    use super::*;
    #[test]
    fn 保持中の左ボタンだけ開始し無効入力や再押下では上書きしない() {
        let model = held();
        let mut input = StickInput::default();
        let now = Duration::from_secs(10);
        for button in [
            None,
            Some(MouseButton::Secondary),
            Some(MouseButton::Auxiliary),
        ] {
            assert!(!input.press(&model, 1, button, Some([0.0, 1.0]), now));
        }
        for target in [None, Some([f32::NAN, 1.0])] {
            assert!(!input.press(&model, 1, Some(MouseButton::Primary), target, now));
        }
        assert_eq!(input.pointer_id(), None);
        assert!(input.press(&model, 1, Some(MouseButton::Primary), Some([0.0, 1.0]), now));
        assert!(!input.press(&model, 2, Some(MouseButton::Primary), Some([0.2, 0.2]), now));
        assert!(!input.press(&model, 1, Some(MouseButton::Primary), Some([0.2, 0.2]), now));
        assert_eq!(input.preview(model, now).target(), Some([0.0, 1.0]));
        let moving = GuiStick::r#move(GuiStick::new(*model.as_ref(), Some([0.0, 1.0]), 6.0));
        assert!(!StickInput::default().press(
            &moving,
            1,
            Some(MouseButton::Primary),
            Some([0.0, 1.0]),
            now
        ));
    }
    #[test]
    fn メーターが往復しても保持状態と零実速度を保ち解放時の表示速度で一度だけ投げる() {
        let model = held();
        let mut input = StickInput::default();
        let start = Duration::from_secs(10);
        assert!(input.press(
            &model,
            1,
            Some(MouseButton::Primary),
            Some([0.0, 1.0]),
            start
        ));
        for (ms, speed) in [
            (0, 0.0),
            (500, 5.0),
            (1000, 10.0),
            (1500, 5.0),
            (2000, 0.0),
            (2500, 5.0),
        ] {
            let preview = input.preview(model, start + Duration::from_millis(ms));
            assert_eq!(preview.speed(), speed);
            assert_eq!(preview.velocity(), [0.0; 2]);
            assert_eq!(preview.position(), model.position());
            assert_eq!(preview.as_ref().state(), StickState::Held);
            assert_eq!(preview.as_ref().id(), model.as_ref().id());
        }
        assert_eq!(input.preview(model, Duration::ZERO).speed(), 0.0);
        let now = start + Duration::from_millis(500);
        let preview = input.preview(model, now);
        assert_eq!(
            input.release(model, 2, Some(MouseButton::Primary), now),
            model
        );
        assert_eq!(
            input.release(model, 1, Some(MouseButton::Secondary), now),
            model
        );
        input.move_to(1, None);
        let moved = input.release(model, 1, Some(MouseButton::Primary), now);
        assert_eq!(moved.speed(), preview.speed());
        assert_eq!(moved.target(), preview.target());
        assert_eq!(moved.as_ref().state(), StickState::Move);
        assert_eq!(moved.velocity(), [0.0, 5.0]);
        assert_eq!(input.pointer_id(), None);
        assert_eq!(
            input.release(moved, 1, Some(MouseButton::Primary), now),
            moved
        );
        assert_eq!(input.preview(moved, now), moved);
    }
    #[test]
    fn 所有ポインターの有効な位置だけを照準に反映する() {
        let model = held();
        let mut input = StickInput::default();
        input.move_to(1, Some([1.0, 1.0]));
        assert_eq!(input.preview(model, Duration::ZERO), model);
        input.press(
            &model,
            1,
            Some(MouseButton::Primary),
            Some([0.0, 1.0]),
            Duration::ZERO,
        );
        input.move_to(2, Some([0.2, 0.2]));
        assert_eq!(
            input.preview(model, Duration::ZERO).target(),
            Some([0.0, 1.0])
        );
        input.move_to(1, Some([0.1, 0.5]));
        input.move_to(1, Some([f32::INFINITY, 0.5]));
        input.move_to(1, None);
        assert_eq!(
            input.preview(model, Duration::ZERO).target(),
            Some([0.1, 0.5])
        );
    }
    #[test]
    fn 取消や無効方向や零速度では投げず次の入力を受け付ける() {
        let model = held();
        let mut input = StickInput::default();
        input.press(
            &model,
            1,
            Some(MouseButton::Primary),
            Some([0.0, 1.0]),
            Duration::ZERO,
        );
        assert!(!input.cancel(Some(2)));
        assert_eq!(input.pointer_id(), Some(1));
        assert!(input.cancel(Some(1)));
        assert_eq!(
            input.release(
                model,
                1,
                Some(MouseButton::Primary),
                Duration::from_millis(500)
            ),
            model
        );
        assert!(input.press(
            &model,
            1,
            Some(MouseButton::Primary),
            Some([0.0, 1.0]),
            Duration::ZERO
        ));
        assert!(input.cancel(None));
        for (target, time) in [([0.0; 2], 500), ([0.0, 1.0], 0)] {
            input.press(
                &model,
                1,
                Some(MouseButton::Primary),
                Some(target),
                Duration::ZERO,
            );
            let result = input.release(
                model,
                1,
                Some(MouseButton::Primary),
                Duration::from_millis(time),
            );
            assert_eq!(result, model);
        }
        assert!(!input.cancel(None));
    }
}
