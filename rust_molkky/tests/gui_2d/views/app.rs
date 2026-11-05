use super::*;
use crate::{Move, Name, ScoreBoard, ScoreState, Skittle};
use support::*;

pub(in crate::gui_2d::views) mod support {
    use super::*;
    use std::time::Duration;

    pub struct Scene {
        pub teams: Vec<Team>,
        pub field: GuiField,
        pub stick: GuiStick,
        pub skittles: Vec<GuiSkittle>,
        pub rules: GuiMolkky,
        pub accumulator: Duration,
    }
    impl Scene {
        pub fn new(mut teams: Vec<Team>) -> Self {
            let (field, stick, skittles) = create_field();
            let rules = create_world(&stick, &skittles);
            for team in &mut teams {
                *team = team.with_turn(false);
            }
            advance_turn(&mut teams, &rules);
            Self {
                teams,
                field,
                stick,
                skittles,
                rules,
                accumulator: Duration::ZERO,
            }
        }
        pub fn frame(&mut self, elapsed: Duration) -> bool {
            if !self.teams.iter().any(Team::turn) {
                return false;
            }
            let done = super::super::super::field::advance(
                &mut self.rules,
                &mut self.stick,
                &mut self.skittles,
                elapsed,
                &mut self.accumulator,
            );
            if done {
                finish_throw(
                    &mut self.teams,
                    &mut self.stick,
                    &mut self.skittles,
                    &mut self.rules,
                );
            }
            done
        }
        pub fn start(&mut self, target: [f32; 2], speed: f32) {
            self.stick = GuiStick::r#move(GuiStick::new(*self.stick.as_ref(), Some(target), speed));
        }
    }

    pub fn team(name: &str, points: &[i32]) -> Team {
        let rules = create_world(&GuiStick::default(), &[]);
        let history = points
            .iter()
            .map(|&point| {
                rules.calc_score(&if point == 0 {
                    vec![]
                } else {
                    vec![Skittle::new(point)]
                })
            })
            .collect();
        Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
    }
    pub fn render_scene(scene: &Scene) -> String {
        type Snapshot = (Vec<Team>, GuiField, GuiStick, Vec<GuiSkittle>);
        fn fixture((teams, field, stick, skittles): Snapshot) -> Element {
            let (teams, stick, skittles, rules) = use_hook(|| {
                let rules = create_world(&stick, &skittles);
                (
                    Signal::new(teams),
                    Signal::new(stick),
                    Signal::new(skittles),
                    Signal::new(rules),
                )
            });
            let enabled = use_memo(move || teams.read().iter().any(Team::turn));
            super::super::render(teams, field, stick, skittles, rules, enabled.into())
        }
        let mut dom = VirtualDom::new_with_props(
            fixture,
            (
                scene.teams.clone(),
                scene.field.clone(),
                scene.stick,
                scene.skittles.clone(),
            ),
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

fn record(scene: &mut Scene, numbers: &[i32]) {
    scene.start([0.0, 1.0], 6.0);
    scene.skittles = scene
        .skittles
        .iter()
        .map(|pin| {
            let state = if numbers.contains(&pin.as_ref().number()) {
                SkittleState::Fall
            } else {
                SkittleState::Stand
            };
            pin.update(pin.position(), pin.angle(), state, false)
        })
        .collect();
    scene.stick = scene
        .stick
        .update(scene.stick.position(), scene.stick.angle(), [0.0; 2], 0.0);
    finish_throw(
        &mut scene.teams,
        &mut scene.stick,
        &mut scene.skittles,
        &mut scene.rules,
    );
}

mod scoring {
    use super::*;

    #[test]
    fn 複数本は本数で一度だけ採点して交代し個体とフィールドを保つ() {
        let mut game = Scene::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = game.field.as_ref().id();
        let stick_id = game.stick.as_ref().id();
        let original: Vec<_> = game
            .skittles
            .iter()
            .map(|skittle| (skittle.as_ref().id(), skittle.position()))
            .collect();
        record(&mut game, &[5, 8]);
        assert_eq!(game.teams[0].score_board().total(), 2);
        assert_eq!(game.teams[0].score_board().history().len(), 1);
        assert!(game.teams[1].turn());
        assert!(game.teams[1].score_board().history().is_empty());
        assert!(
            game.skittles
                .iter()
                .all(|skittle| skittle.as_ref().state() == SkittleState::Stand)
        );
        assert_eq!(
            game.skittles
                .iter()
                .map(|skittle| (skittle.as_ref().id(), skittle.position()))
                .collect::<Vec<_>>(),
            original
        );
        assert_eq!(game.field.as_ref().id(), field_id);
        assert_eq!(game.stick.as_ref().id(), stick_id);
        finish_throw(
            &mut game.teams,
            &mut game.stick,
            &mut game.skittles,
            &mut game.rules,
        );
        game.frame(std::time::Duration::from_millis(20));
        assert_eq!(game.teams[0].score_board().history().len(), 1);
    }

    #[test]
    fn 超過は二十五点へ戻して次チームへ進む() {
        let mut game = Scene::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        record(&mut game, &[2]);
        assert_eq!(game.teams[0].score_board().total(), 25);
        assert_eq!(game.stick.as_ref().state(), StickState::Held);
        assert!(game.teams[1].turn());
    }

    #[test]
    fn 五十点なら終了して次の手番や追加の履歴を作らない() {
        let mut game = Scene::new(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        record(&mut game, &[1]);
        assert!(!game.teams.iter().any(Team::turn));
        assert_eq!(
            game.rules.judge_game(&game.teams).unwrap().id(),
            game.teams[0].id()
        );
        assert!(game.teams.iter().all(|team| !team.turn()));
        game.frame(std::time::Duration::from_millis(20));
        assert_eq!(game.stick.target(), None);
        assert_eq!(game.teams[0].score_board().history().len(), 6);
    }

    #[test]
    fn 三連続ミスで失格し最後の生存者が履歴を増やさず五十点で勝つ() {
        let mut game = Scene::new(vec![team("赤", &[0, 0]), team("青", &[])]);
        record(&mut game, &[]);
        assert!(!game.teams.iter().any(Team::turn));
        assert_eq!(
            game.teams[0].score_board().state(),
            ScoreState::Disqualified
        );
        assert_eq!(game.teams[1].score_board().total(), 50);
        assert!(game.teams[1].score_board().history().is_empty());
    }

    #[test]
    fn 単独の失格は勝者なしで終了する() {
        let mut game = Scene::new(vec![team("赤", &[0, 0])]);
        record(&mut game, &[]);
        assert!(!game.teams.iter().any(Team::turn));
        assert!(game.rules.judge_game(&game.teams).is_none());
    }
}

mod composition {
    use super::*;
    #[test]
    fn アプリのモデルを表示し描画中は物理や状態を更新しない() {
        let mut scene = Scene::new(vec![team("赤", &[]), team("青", &[])]);
        let field_id = scene.field.as_ref().id();
        let pin = scene.skittles[0];
        scene.skittles[0] = pin.update(pin.position(), pin.angle(), SkittleState::Fall, false);
        let html = render_scene(&scene);
        assert_eq!(html.matches("data-state=\"stand\"").count(), 11);
        assert_eq!(html.matches("data-state=\"fall\"").count(), 1);
        assert!(html.contains(&field_id.to_string()));
        assert!(html.contains("data-phase=\"ready\""));
        assert!(html.contains("得点表") && html.contains("<meter") && html.contains("モルック"));
        assert!(scene.teams[0].turn() && !scene.teams[1].turn());
        assert_eq!(html, render_scene(&scene));
    }
    #[test]
    fn 勝利と不戦勝と勝者なしの終了を表示する() {
        for (teams, expected) in [
            (vec![team("赤", &[10; 5]), team("青", &[])], "赤の勝利"),
            (vec![team("赤", &[0; 3]), team("青", &[])], "青の勝利"),
            (vec![team("赤", &[0; 3])], "勝者なし"),
            (vec![], "勝者なし"),
        ] {
            let scene = Scene::new(teams);
            let html = render_scene(&scene);
            assert!(html.contains(expected), "{html}");
            assert!(html.contains("data-phase=\"finished\""));
        }
    }
}
