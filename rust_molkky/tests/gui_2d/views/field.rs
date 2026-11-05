use super::super::app::tests::support::*;
use super::*;
use crate::gui_2d::services::field::{create_field, create_world};
use crate::{Move, Skittle as CoreSkittle};

mod composition {
    use super::*;
    #[test]
    fn 範囲を保って上向きの座標と子要素を表示する() {
        fn fixture() -> Element {
            field_shape(
                [-2.0, 0.0, 4.0, 8.0],
                rsx! { circle { cx: "1", cy: "2", r: "0.1" } },
                EventHandler::new(|_| {}),
            )
        }
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("viewBox=\"-2 -8 4 8\""), "{html}");
        assert!(html.contains("scale(1 -1)") && html.contains("<circle"));
        assert!(html.contains("モルックのフィールド"));
    }
}

mod simulation {
    use super::*;

    #[test]
    fn 棒が停止しても命中中のスキットルが残る間は採点しない() {
        let mut game = Scene::new(vec![team("赤", &[]), team("青", &[])]);
        game.start([0.0, 1.0], 6.0);
        game.stick = game.stick.update([0.0; 2], 0.0, [0.0; 2], 0.0);
        game.skittles = vec![GuiSkittle::new(
            CoreSkittle::new(5).with_state(SkittleState::Hit),
            [0.0, 0.7],
        )];
        game.rules = create_world(&game.stick, &game.skittles);
        game.frame(Duration::from_millis(20));
        assert_eq!(game.stick.as_ref().state(), StickState::Stop);
        assert_eq!(game.skittles[0].as_ref().state(), SkittleState::Hit);
        assert_ne!(game.stick.as_ref().state(), StickState::Held);
        assert!(game.teams[0].score_board().history().is_empty());
        for _ in 0..1200 {
            game.frame(Duration::from_millis(20));
            if game.stick.as_ref().state() == StickState::Held {
                break;
            }
        }
        assert_eq!(game.stick.as_ref().state(), StickState::Held);
        assert_eq!(game.teams[0].score_board().total(), 0);
        assert_eq!(game.teams[0].score_board().history().len(), 1);
        assert!(game.teams[1].turn());
        game.frame(Duration::from_millis(20));
        assert_eq!(game.teams[0].score_board().history().len(), 1);
    }
}

mod contact_routing {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn 同番号の連鎖でも配置順によらず双方を更新し一度だけ二点を採点する() {
        for positions in [
            vec![[0.02, 0.7], [0.06, 0.85]],
            vec![[0.06, 0.85], [0.02, 0.7]],
        ] {
            let (field, stick, _) = create_field();
            let skittles: Vec<_> = positions
                .into_iter()
                .map(|position| GuiSkittle::new(CoreSkittle::new(12), position))
                .collect();
            let ids: HashSet<_> = skittles.iter().map(|s| s.as_ref().id()).collect();
            let mut game = Scene::new(vec![team("赤", &[]), team("青", &[])]);
            game.field = field;
            game.stick = stick;
            game.skittles = skittles;
            game.rules = create_world(&game.stick, &game.skittles);
            game.start([0.0, 1.0], 6.0);
            let mut hit_ids = HashSet::new();
            for _ in 0..1200 {
                game.frame(Duration::from_millis(20));
                hit_ids.extend(
                    game.skittles
                        .iter()
                        .filter(|s| s.as_ref().state() == SkittleState::Hit)
                        .map(|s| s.as_ref().id()),
                );
                if game.stick.as_ref().state() == StickState::Held {
                    break;
                }
            }
            assert_eq!(hit_ids, ids);
            assert_eq!(game.stick.as_ref().state(), StickState::Held);
            assert_eq!(game.teams[0].score_board().total(), 2);
            assert_eq!(game.teams[0].score_board().history().len(), 1);
            assert!(game.teams[1].turn());
            assert!(
                game.skittles
                    .iter()
                    .all(|s| s.as_ref().state() == SkittleState::Stand)
            );
            game.frame(Duration::from_millis(20));
            assert_eq!(game.teams[0].score_board().history().len(), 1);
            assert_eq!(
                game.skittles
                    .iter()
                    .map(|s| s.as_ref().id())
                    .collect::<HashSet<_>>(),
                ids
            );
        }
    }
}

mod fixed_updates {
    use super::*;
    #[test]
    fn 固定刻み未満は進めず蓄力中の正速度でも物理を進めない() {
        let mut scene = Scene::new(vec![team("赤", &[]), team("青", &[])]);
        scene.stick = GuiStick::new(*scene.stick.as_ref(), Some([0.0, 1.0]), 6.0);
        let held = scene.stick;
        assert!(!scene.frame(Duration::from_secs(1)));
        assert_eq!(scene.stick, held);
        assert!(scene.teams[0].score_board().history().is_empty());
        scene.stick = GuiStick::r#move(held);
        assert!(!scene.frame(Duration::from_millis(1)));
        assert_eq!(scene.stick.position(), held.position());
        scene.frame(Duration::from_millis(20));
        assert_ne!(scene.stick.position(), held.position());
        assert!(scene.stick.speed() < 6.0);
    }
    #[test]
    fn 高速の反射中も固定範囲を保ち停止後一度採点し位置と個体を保持する() {
        let mut scene = Scene::new(vec![team("赤", &[]), team("青", &[])]);
        let id = scene.stick.as_ref().id();
        let field = scene.field.as_ref().id();
        scene.start([0.25, 0.0], 10.0);
        let initial = render_scene(&scene);
        let bounds = initial
            .split("viewBox=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(bounds, "-0.3 -1.35 0.6 1.5");
        // タブ復帰相当の時間差でも一度で停止させない。
        scene.frame(Duration::from_secs(30));
        assert_eq!(scene.stick.as_ref().state(), StickState::Move);
        for _ in 0..1200 {
            let done = scene.frame(Duration::from_millis(20));
            assert!(render_scene(&scene).contains(&format!("viewBox=\"{bounds}\"")));
            if done {
                break;
            }
        }
        assert_eq!(scene.stick.as_ref().state(), StickState::Held);
        assert_eq!(scene.teams[0].score_board().total(), 0);
        assert_eq!(scene.teams[0].score_board().history().len(), 1);
        assert!(scene.teams[1].turn());
        scene.frame(Duration::from_secs(30));
        assert_eq!(scene.teams[0].score_board().history().len(), 1);
        assert_eq!(scene.stick.as_ref().id(), id);
        assert_eq!(scene.field.as_ref().id(), field);
    }
}
