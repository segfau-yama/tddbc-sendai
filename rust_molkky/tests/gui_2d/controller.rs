use super::*;

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
        let mut input = Controller::default();
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
        let mut input = Controller::default();
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
        let mut input = Controller::default();
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
        let mut input = Controller::default();
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
        let mut input = Controller::default();
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
