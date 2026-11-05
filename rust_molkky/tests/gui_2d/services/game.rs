use super::*;
use crate::gui_2d::services::field::create_field;
use crate::{Name, ScoreBoard, ScoreState};

fn team(name: &str, points: &[i32]) -> Team {
    let rules = create_world(&GuiStick::default(), &[]);
    let history = points
        .iter()
        .map(|&value| {
            let skittles = if value == 0 {
                vec![]
            } else {
                vec![crate::Skittle::new(value)]
            };
            rules.calc_score(&skittles)
        })
        .collect();
    Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
}

fn game(teams: Vec<Team>) -> Game {
    let (field, stick, skittles) = create_field();
    Game::new(teams, field, stick, skittles)
}

fn record(game: &mut Game, numbers: &[i32]) {
    game.aim([0.0, 1.0], 6.0);
    assert!(game.throw());
    game.skittles = game
        .skittles
        .iter()
        .map(|skittle| {
            let state = if numbers.contains(&skittle.as_ref().number()) {
                SkittleState::Fall
            } else {
                SkittleState::Stand
            };
            skittle.update(skittle.position(), skittle.angle(), state, false)
        })
        .collect();
    game.stick = game
        .stick
        .update(game.stick.position(), game.stick.angle(), [0.0; 2], 0.0);
    game.finish_throw();
}

mod initialization {
    use super::*;

    #[test]
    fn 注入した個体と共通フィールドと棒を保持して先頭の手番から始める() {
        let teams = vec![team("赤", &[]), team("青", &[]).with_turn(true)];
        let team_ids: Vec<_> = teams.iter().map(Team::id).collect();
        let (field, stick, skittles) = create_field();
        let field_id = field.as_ref().id();
        let original = skittles.clone();
        let game = Game::new(teams, field, stick, skittles);
        assert_eq!(
            game.teams().iter().map(Team::id).collect::<Vec<_>>(),
            team_ids
        );
        assert!(game.teams()[0].turn());
        assert!(!game.teams()[1].turn());
        assert_eq!(game.field().as_ref().id(), field_id);
        assert_eq!(*game.stick(), stick);
        assert_eq!(game.skittles(), original);
        assert!(!game.is_simulating());
        assert!(!game.is_finished());
    }

    #[test]
    fn 参加者がいなければ終了して投擲や更新を受け付けない() {
        let mut game = game(vec![]);
        game.aim([0.0, 1.0], 6.0);
        assert!(!game.throw());
        game.update();
        assert!(game.is_finished());
        assert!(game.winner().is_none());
        assert_eq!(game.stick().target(), None);
    }
}

mod scoring {
    use super::*;

    #[test]
    fn 複数本は本数で一度だけ採点して交代し個体とフィールドを保つ() {
        let mut game = game(vec![team("赤", &[]), team("青", &[])]);
        let field_id = game.field().as_ref().id();
        let stick_id = game.stick().as_ref().id();
        let original: Vec<_> = game
            .skittles()
            .iter()
            .map(|skittle| (skittle.as_ref().id(), skittle.position()))
            .collect();
        record(&mut game, &[5, 8]);
        assert_eq!(game.teams()[0].score_board().total(), 2);
        assert_eq!(game.teams()[0].score_board().history().len(), 1);
        assert!(game.teams()[1].turn());
        assert!(game.teams()[1].score_board().history().is_empty());
        assert!(
            game.skittles()
                .iter()
                .all(|skittle| skittle.as_ref().state() == SkittleState::Stand)
        );
        assert_eq!(
            game.skittles()
                .iter()
                .map(|skittle| (skittle.as_ref().id(), skittle.position()))
                .collect::<Vec<_>>(),
            original
        );
        assert_eq!(game.field().as_ref().id(), field_id);
        assert_eq!(game.stick().as_ref().id(), stick_id);
        game.finish_throw();
        game.update();
        assert_eq!(game.teams()[0].score_board().history().len(), 1);
    }

    #[test]
    fn 超過は二十五点へ戻して次チームへ進む() {
        let mut game = game(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        record(&mut game, &[2]);
        assert_eq!(game.teams()[0].score_board().total(), 25);
        assert!(!game.is_simulating());
        assert!(game.teams()[1].turn());
    }

    #[test]
    fn 五十点なら終了して以降の照準と投擲と更新を無効にする() {
        let mut game = game(vec![team("赤", &[12, 12, 12, 12, 1]), team("青", &[])]);
        record(&mut game, &[1]);
        assert!(game.is_finished());
        assert_eq!(game.winner().unwrap().id(), game.teams()[0].id());
        assert!(game.teams().iter().all(|team| !team.turn()));
        game.aim([0.0, 1.0], 6.0);
        assert!(!game.throw());
        game.update();
        assert_eq!(game.stick().target(), None);
        assert_eq!(game.teams()[0].score_board().history().len(), 6);
    }

    #[test]
    fn 三連続ミスで失格し最後の生存者が履歴を増やさず五十点で勝つ() {
        let mut game = game(vec![team("赤", &[0, 0]), team("青", &[])]);
        record(&mut game, &[]);
        assert!(game.is_finished());
        assert_eq!(
            game.teams()[0].score_board().state(),
            ScoreState::Disqualified
        );
        assert_eq!(game.teams()[1].score_board().total(), 50);
        assert!(game.teams()[1].score_board().history().is_empty());
    }

    #[test]
    fn 単独の失格は勝者なしで終了する() {
        let mut game = game(vec![team("赤", &[0, 0])]);
        record(&mut game, &[]);
        assert!(game.is_finished());
        assert!(game.winner().is_none());
    }
}

mod simulation {
    use super::*;

    #[test]
    fn 棒が停止しても命中中のスキットルが残る間は採点しない() {
        let mut game = game(vec![team("赤", &[]), team("青", &[])]);
        game.aim([0.0, 1.0], 6.0);
        assert!(game.throw());
        game.stick = game.stick.update([0.0; 2], 0.0, [0.0; 2], 0.0);
        game.skittles = vec![GuiSkittle::new(
            crate::Skittle::new(5).with_state(SkittleState::Hit),
            [0.0, 0.7],
        )];
        game.rules = create_world(&game.stick, &game.skittles);
        game.update();
        assert_eq!(game.stick().as_ref().state(), StickState::Stop);
        assert_eq!(game.skittles()[0].as_ref().state(), SkittleState::Hit);
        assert!(game.is_simulating());
        assert!(game.teams()[0].score_board().history().is_empty());
        for _ in 0..1200 {
            game.update();
            if !game.is_simulating() {
                break;
            }
        }
        assert!(!game.is_simulating());
        assert_eq!(game.teams()[0].score_board().total(), 0);
        assert_eq!(game.teams()[0].score_board().history().len(), 1);
        assert!(game.teams()[1].turn());
        game.update();
        assert_eq!(game.teams()[0].score_board().history().len(), 1);
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
                .map(|position| GuiSkittle::new(crate::Skittle::new(12), position))
                .collect();
            let ids: HashSet<_> = skittles.iter().map(|s| s.as_ref().id()).collect();
            let mut game = Game::new(
                vec![team("赤", &[]), team("青", &[])],
                field,
                stick,
                skittles,
            );
            game.aim([0.0, 1.0], 6.0);
            assert!(game.throw());
            let mut hit_ids = HashSet::new();
            for _ in 0..1200 {
                game.update();
                hit_ids.extend(
                    game.skittles()
                        .iter()
                        .filter(|s| s.as_ref().state() == SkittleState::Hit)
                        .map(|s| s.as_ref().id()),
                );
                if !game.is_simulating() {
                    break;
                }
            }
            assert_eq!(hit_ids, ids);
            assert!(!game.is_simulating());
            assert_eq!(game.teams()[0].score_board().total(), 2);
            assert_eq!(game.teams()[0].score_board().history().len(), 1);
            assert!(game.teams()[1].turn());
            assert!(
                game.skittles()
                    .iter()
                    .all(|s| s.as_ref().state() == SkittleState::Stand)
            );
            game.update();
            assert_eq!(game.teams()[0].score_board().history().len(), 1);
            assert_eq!(
                game.skittles()
                    .iter()
                    .map(|s| s.as_ref().id())
                    .collect::<HashSet<_>>(),
                ids
            );
        }
    }
}
