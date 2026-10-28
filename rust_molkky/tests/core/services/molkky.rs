use rust_molkky::{GameStatus, Molkky, Name, Skittle, Team};

#[test]
fn 空の実装でも現在の採点と失格と最後の生存チームの勝利を使える() {
    struct Rules;
    impl Molkky for Rules {}
    let rules = Rules;
    assert_eq!(rules.calc_score(&[Skittle::new(12)]).value(), 12);
    assert_eq!(
        rules
            .calc_score(&[Skittle::new(12), Skittle::new(12)])
            .value(),
        2
    );
    let mut teams = vec![
        Team::new(Name::new("赤").unwrap()),
        Team::new(Name::new("青").unwrap()),
    ];
    assert_eq!(rules.judge_game(&teams), GameStatus::InProgress);
    assert_eq!(rules.next_team(&teams, 0), Some(1));
    for _ in 0..3 {
        teams = rules.record_point(&teams, 0, rules.calc_score(&[]));
    }
    assert_eq!(
        rules.judge_game(&teams),
        GameStatus::Finished { winner: Some(1) }
    );
    assert_eq!(rules.next_team(&teams, 0), None);
}

#[test]
fn 上書きした判定を次チームと記録の既定実装が利用する() {
    struct Finished;
    impl Molkky for Finished {
        fn judge_game(&self, _: &[Team]) -> GameStatus {
            GameStatus::Finished { winner: None }
        }
    }
    let rules: &dyn Molkky = &Finished;
    let teams = vec![Team::new(Name::new("赤").unwrap())];
    assert_eq!(rules.next_team(&teams, 0), None);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rules.record_point(
            &teams,
            0,
            rules.calc_score(&[])
        )))
        .is_err()
    );
    assert!(teams[0].score_board().history().is_empty());
}

mod judging {
    use rust_molkky::Name;
    use rust_molkky::{GameStatus, Molkky, ScoreBoard, SimpleMolkky, Skittle, Team};

    fn team(name: &str, points: &[i32]) -> Team {
        let board = points.iter().fold(ScoreBoard::new(), |board, &number| {
            let pins = if number == 0 {
                vec![]
            } else {
                vec![Skittle::new(number)]
            };
            board.add(SimpleMolkky.calc_score(&pins))
        });
        Team::new(Name::new(name).unwrap()).with_score_board(board)
    }

    #[test]
    fn 五十点に到達したチームを勝者とする() {
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[]), team("青", &[10; 5])]),
            GameStatus::Finished { winner: Some(1) }
        );
    }

    #[test]
    fn 元の参加者が複数なら最後の生存チームが零点でも勝つ() {
        let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[0; 3])];
        assert_eq!(
            SimpleMolkky.judge_game(&teams),
            GameStatus::Finished { winner: Some(1) }
        );
        assert_eq!(teams[1].score_board().value(), 0);
    }

    #[test]
    fn 単独参加は五十点まで進行し失格なら勝者なしで終わる() {
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[])]),
            GameStatus::InProgress
        );
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[0; 3])]),
            GameStatus::Finished { winner: None }
        );
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[10; 5])]),
            GameStatus::Finished { winner: Some(0) }
        );
    }

    #[test]
    fn 複数チームが残る間は進行し全員失格なら勝者なしになる() {
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[0; 3]), team("青", &[12; 4]), team("緑", &[])]),
            GameStatus::InProgress
        );
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[0; 3]), team("青", &[0; 3])]),
            GameStatus::Finished { winner: None }
        );
        assert_eq!(
            SimpleMolkky.judge_game(&[team("赤", &[12, 12, 12, 12, 3]), team("青", &[])]),
            GameStatus::InProgress
        );
    }

    #[test]
    #[should_panic(expected = "チームは1件以上必要です")]
    fn 空の入力を拒否する() {
        SimpleMolkky.judge_game(&[]);
    }

    #[test]
    #[should_panic(expected = "勝者が複数います")]
    fn 複数の五十点到達チームを拒否する() {
        SimpleMolkky.judge_game(&[team("赤", &[10; 5]), team("青", &[10; 5])]);
    }

    #[test]
    fn 終了後の不正な履歴を独立した判定関数でも拒否する() {
        for points in [&[0, 0, 0, 1][..], &[10, 10, 10, 10, 10, 1][..]] {
            assert!(
                std::panic::catch_unwind(|| SimpleMolkky.judge_game(&[team("赤", points)]))
                    .is_err()
            );
        }
    }

    #[test]
    fn 次チーム判定は失格者を飛ばし終了後は返さない() {
        let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[])];
        assert_eq!(SimpleMolkky.next_team(&teams, 2), Some(1));
        assert_eq!(SimpleMolkky.next_team(&[team("赤", &[])], 0), Some(0));
        assert_eq!(
            SimpleMolkky.next_team(&[team("赤", &[0; 3]), team("青", &[])], 0),
            None
        );
    }

    #[test]
    fn 記録関数は元の履歴を変えず終了という共通状態を返す() {
        let original = [team("赤", &[0, 0]), team("青", &[])];
        let updated = SimpleMolkky.record_point(&original, 0, SimpleMolkky.calc_score(&[]));
        assert_eq!(
            SimpleMolkky.judge_game(&updated),
            GameStatus::Finished { winner: Some(1) }
        );
        assert_eq!(original[0].score_board().history().len(), 2);
    }

    #[test]
    fn 複数チームを保持し開始時は進行中になる() {
        let game = vec![
            Team::new(Name::new("赤").unwrap()),
            Team::new(Name::new("青").unwrap()),
        ];
        assert_eq!(game.len(), 2);
        assert_eq!(game[0].name().as_str(), "赤");
        assert_eq!(game[1].name().as_str(), "青");
        assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
    }

    #[test]
    fn 同名のチームでも指定したチームだけに記録する() {
        let mut game = vec![
            Team::new(Name::new("同じ名前").unwrap()),
            Team::new(Name::new("同じ名前").unwrap()),
        ];
        let point = SimpleMolkky.calc_score(&[Skittle::new(5)]);
        assert_eq!(point.value(), 5);
        game = SimpleMolkky.record_point(&game, 1, point);
        assert_eq!(game[0].score_board().value(), 0);
        assert!(game[0].score_board().history().is_empty());
        assert_eq!(game[1].score_board().value(), 5);
        assert_eq!(
            game[1]
                .score_board()
                .history()
                .iter()
                .map(|point| point.value())
                .collect::<Vec<_>>(),
            &[5]
        );
    }

    fn game_at_forty_nine() -> Vec<Team> {
        let mut game = vec![
            Team::new(Name::new("赤").unwrap()),
            Team::new(Name::new("青").unwrap()),
        ];
        let twelve = SimpleMolkky.calc_score(&[Skittle::new(12)]);
        for _ in 0..4 {
            game = SimpleMolkky.record_point(&game, 1, twelve);
        }
        game = SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(1)]));
        game
    }

    mod invalid_operations {
        use super::*;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        #[test]
        fn 五十点到達後に追加記録がある履歴は判定時に拒否する() {
            let ten = SimpleMolkky.calc_score(&[Skittle::new(10)]);
            let mut board = rust_molkky::ScoreBoard::new();
            for _ in 0..5 {
                board = board.add(ten);
            }
            for point in [
                SimpleMolkky.calc_score(&[]),
                SimpleMolkky.calc_score(&[Skittle::new(1)]),
            ] {
                let invalid =
                    Team::new(Name::new("赤").unwrap()).with_score_board(board.add(point));
                assert!(catch_unwind(|| SimpleMolkky.judge_game(&[invalid])).is_err());
            }
            assert_eq!(board.value(), 50);
            assert_eq!(board.history().len(), 5);
        }

        #[test]
        #[should_panic(expected = "チームは1件以上必要です")]
        fn チームが空の試合は判定できない() {
            SimpleMolkky.judge_game(&[]);
        }

        #[test]
        #[should_panic(expected = "勝者が複数います")]
        fn 勝者が複数いる状態は判定時に拒否する() {
            let mut red = Team::new(Name::new("赤").unwrap());
            let mut blue = Team::new(Name::new("青").unwrap());
            let ten = SimpleMolkky.calc_score(&[Skittle::new(10)]);
            for _ in 0..5 {
                red = red.with_score_board(red.score_board().add(ten));
                blue = blue.with_score_board(blue.score_board().add(ten));
            }
            SimpleMolkky.judge_game(&[red, blue]);
        }

        #[test]
        fn 存在しないチームへの記録を拒否して得点を保つ() {
            let mut game = vec![
                Team::new(Name::new("赤").unwrap()),
                Team::new(Name::new("青").unwrap()),
            ];
            game = SimpleMolkky.record_point(&game, 0, SimpleMolkky.calc_score(&[Skittle::new(5)]));
            for index in [2, usize::MAX] {
                let failed = catch_unwind(AssertUnwindSafe(|| {
                    game = SimpleMolkky.record_point(
                        &game,
                        index,
                        SimpleMolkky.calc_score(&[Skittle::new(12)]),
                    );
                }));
                assert!(failed.is_err());
                assert_eq!(
                    game[0]
                        .score_board()
                        .history()
                        .iter()
                        .map(|point| point.value())
                        .collect::<Vec<_>>(),
                    &[5]
                );
                assert!(game[1].score_board().history().is_empty());
                assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            }
        }

        #[test]
        fn 終了後はどのチームへの記録も拒否して履歴と勝者を保つ() {
            let mut game = game_at_forty_nine();
            game = SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(1)]));
            for index in [0, 1] {
                let failed = catch_unwind(AssertUnwindSafe(|| {
                    game = SimpleMolkky.record_point(
                        &game,
                        index,
                        SimpleMolkky.calc_score(&[Skittle::new(12)]),
                    );
                }));
                assert!(failed.is_err());
                assert!(game[0].score_board().history().is_empty());
                assert_eq!(
                    game[1]
                        .score_board()
                        .history()
                        .iter()
                        .map(|point| point.value())
                        .collect::<Vec<_>>(),
                    &[12, 12, 12, 12, 1, 1]
                );
                assert_eq!(game[1].score_board().value(), 50);
                assert_eq!(
                    SimpleMolkky.judge_game(&game),
                    GameStatus::Finished { winner: Some(1) }
                );
            }
        }
    }

    mod target_score {
        use super::*;

        #[test]
        fn 四十九点ではゲームは終了しない() {
            let game = game_at_forty_nine();
            assert_eq!(game[1].score_board().value(), 49);
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
        }

        #[test]
        fn 五十点ちょうどで該当チームの勝利になる() {
            let mut game = game_at_forty_nine();
            game = SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(1)]));
            assert_eq!(game[1].score_board().value(), 50);
            assert_eq!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { winner: Some(1) }
            );
        }

        #[test]
        fn 超過時は25点に戻してゲームを継続する() {
            let mut game = game_at_forty_nine();
            let point = SimpleMolkky.calc_score(&[Skittle::new(2)]);
            assert_eq!(point.value(), 2);
            game = SimpleMolkky.record_point(&game, 1, point);
            assert_eq!(game[1].score_board().value(), 25);
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            game =
                SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(12)]));
            assert_eq!(game[1].score_board().value(), 37);
            game = SimpleMolkky.record_point(&game, 0, SimpleMolkky.calc_score(&[Skittle::new(5)]));
            assert_eq!(game[0].score_board().value(), 5);
            game =
                SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(12)]));
            game = SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(1)]));
            assert_eq!(game[1].score_board().value(), 50);
            assert_eq!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { winner: Some(1) }
            );
        }
    }

    #[test]
    fn 一チームのゲームと既に勝者がいるゲームも表現できる() {
        assert_eq!(
            SimpleMolkky.judge_game(&[Team::new(Name::new("赤").unwrap())]),
            GameStatus::InProgress
        );
        let mut winner = Team::new(Name::new("赤").unwrap());
        let ten = SimpleMolkky.calc_score(&[Skittle::new(10)]);
        for _ in 0..5 {
            winner = winner.with_score_board(winner.score_board().add(ten));
        }
        assert_eq!(
            SimpleMolkky.judge_game(&[winner]),
            GameStatus::Finished { winner: Some(0) }
        );
    }

    mod restored_rules {
        use super::*;
        use rust_molkky::ScoreBoard;
        use std::panic::catch_unwind;

        fn team_with_points(name: &str, points: &[i32]) -> Team {
            let mut board = ScoreBoard::new();
            for &number in points {
                let point = if number == 0 {
                    SimpleMolkky.calc_score(&[])
                } else {
                    SimpleMolkky.calc_score(&[Skittle::new(number)])
                };
                board = board.add(point);
            }
            Team::new(Name::new(name).unwrap()).with_score_board(board)
        }

        #[test]
        fn 複数チームから残り一チームになったら勝利する() {
            let game = vec![
                team_with_points("赤", &[0, 0, 0]),
                Team::new(Name::new("青").unwrap()),
            ];
            assert_eq!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { winner: Some(1) }
            );
            assert!(matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            assert_eq!(game[1].score_board().value(), 0);
            assert!(
                catch_unwind(|| SimpleMolkky.record_point(
                    &game,
                    1,
                    SimpleMolkky.calc_score(&[Skittle::new(1)])
                ))
                .is_err()
            );
        }

        #[test]
        fn 記録で残り一チームになった時点で終了し元のゲームは変わらない() {
            let game = vec![
                team_with_points("赤", &[0, 0]),
                Team::new(Name::new("青").unwrap()),
            ];
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            let finished = SimpleMolkky.record_point(&game, 0, SimpleMolkky.calc_score(&[]));
            assert!(matches!(
                SimpleMolkky.judge_game(&finished),
                GameStatus::Finished { .. }
            ));
            assert_eq!(
                SimpleMolkky.judge_game(&finished),
                GameStatus::Finished { winner: Some(1) }
            );
            assert_eq!(finished[1].score_board().value(), 0);
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            assert_eq!(game[0].score_board().history().len(), 2);
            assert!(
                catch_unwind(|| SimpleMolkky.record_point(
                    &finished,
                    1,
                    SimpleMolkky.calc_score(&[Skittle::new(1)])
                ))
                .is_err()
            );
        }

        #[test]
        fn 超過した履歴は25点補正後の得点で判定する() {
            let game = vec![team_with_points("赤", &[12, 12, 12, 12, 3])];
            assert_eq!(game[0].score_board().value(), 25);
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
        }

        #[test]
        fn 失格チームがいても二チーム以上残っていれば継続する() {
            let game = vec![
                team_with_points("赤", &[0, 0, 0]),
                Team::new(Name::new("青").unwrap()),
                Team::new(Name::new("緑").unwrap()),
            ];
            assert!(game[0].score_board().is_disqualified());
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            let updated =
                SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(5)]));
            assert_eq!(updated[1].score_board().value(), 5);
            assert_eq!(game[1].score_board().value(), 0);
        }

        #[test]
        fn 失格後に追加記録のある履歴は判定時に拒否する() {
            let disqualified = team_with_points("赤", &[5, 0, 0, 0]);
            for point in [
                SimpleMolkky.calc_score(&[]),
                SimpleMolkky.calc_score(&[Skittle::new(1)]),
            ] {
                let invalid = disqualified.with_score_board(disqualified.score_board().add(point));
                assert!(catch_unwind(|| SimpleMolkky.judge_game(&[invalid])).is_err());
            }
            assert_eq!(disqualified.score_board().value(), 5);
            assert_eq!(disqualified.score_board().history().len(), 4);
        }

        #[test]
        fn 三回目の0点で失格にし失格チームへの記録だけを拒否する() {
            let game = vec![
                team_with_points("赤", &[5, 0, 0]),
                Team::new(Name::new("青").unwrap()),
                Team::new(Name::new("緑").unwrap()),
            ];
            let disqualified = SimpleMolkky.record_point(&game, 0, SimpleMolkky.calc_score(&[]));
            assert!(disqualified[0].score_board().is_disqualified());
            assert!(!matches!(
                SimpleMolkky.judge_game(&disqualified),
                GameStatus::Finished { .. }
            ));
            assert!(
                catch_unwind(|| SimpleMolkky.record_point(
                    &disqualified,
                    0,
                    SimpleMolkky.calc_score(&[Skittle::new(1)])
                ))
                .is_err()
            );
            let continued = SimpleMolkky.record_point(
                &disqualified,
                1,
                SimpleMolkky.calc_score(&[Skittle::new(11)]),
            );
            assert_eq!(continued[1].score_board().value(), 11);
            assert_eq!(
                continued[0].score_board().history(),
                disqualified[0].score_board().history()
            );
            assert!(!game[0].score_board().is_disqualified());
            assert_eq!(game[0].score_board().history().len(), 3);
        }

        #[test]
        fn 全チーム失格なら勝者なしで終了して記録を拒否する() {
            let game = vec![
                team_with_points("赤", &[0, 0, 0]),
                team_with_points("青", &[1, 0, 0, 0]),
            ];
            assert!(matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            assert_eq!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { winner: None }
            );
            for index in [0, 1] {
                assert!(
                    catch_unwind(|| SimpleMolkky.record_point(
                        &game,
                        index,
                        SimpleMolkky.calc_score(&[])
                    ))
                    .is_err()
                );
            }
            assert_eq!(game[0].score_board().value(), 0);
            assert_eq!(game[1].score_board().value(), 1);
        }

        #[test]
        fn 正の点数で連続0点の回数がリセットされる() {
            let game = vec![team_with_points("赤", &[0, 0, 1, 0, 0])];
            assert!(!game[0].score_board().is_disqualified());
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            let updated = SimpleMolkky.record_point(&game, 0, SimpleMolkky.calc_score(&[]));
            assert!(updated[0].score_board().is_disqualified());
            assert!(matches!(
                SimpleMolkky.judge_game(&updated),
                GameStatus::Finished { .. }
            ));
        }

        #[test]
        fn 二チーム以上残っていても50点到達で勝利する() {
            let game = vec![
                team_with_points("赤", &[0, 0, 0]),
                team_with_points("青", &[12, 12, 12, 12, 1]),
                Team::new(Name::new("緑").unwrap()),
            ];
            assert!(!matches!(
                SimpleMolkky.judge_game(&game),
                GameStatus::Finished { .. }
            ));
            let won =
                SimpleMolkky.record_point(&game, 1, SimpleMolkky.calc_score(&[Skittle::new(1)]));
            assert!(matches!(
                SimpleMolkky.judge_game(&won),
                GameStatus::Finished { .. }
            ));
            assert_eq!(
                SimpleMolkky.judge_game(&won),
                GameStatus::Finished { winner: Some(1) }
            );
            assert_eq!(SimpleMolkky.judge_game(&game), GameStatus::InProgress);
            assert_eq!(game[1].score_board().value(), 49);
        }
    }

    mod immutability {
        use super::*;
        #[test]
        fn ゲームへの記録でも元のゲームと他チームは変わらない() {
            let original = vec![
                Team::new(Name::new("赤").unwrap()),
                Team::new(Name::new("青").unwrap()),
            ];
            let updated = SimpleMolkky.record_point(
                &original,
                0,
                SimpleMolkky.calc_score(&[Skittle::new(11)]),
            );
            assert_eq!(original[0].score_board().value(), 0);
            assert_eq!(original[1].score_board().value(), 0);
            assert_eq!(updated[0].score_board().value(), 11);
            assert_eq!(updated[0].id(), original[0].id());
            assert_eq!(updated[1].score_board().value(), 0);
        }
    }

    #[test]
    fn 次チーム判定は無効なインデックスを拒否し勝者なし終了でも返さない() {
        let teams = [team("赤", &[]), team("青", &[])];
        for current in [2, usize::MAX] {
            assert!(std::panic::catch_unwind(|| SimpleMolkky.next_team(&teams, current)).is_err());
        }
        assert_eq!(SimpleMolkky.next_team(&[team("赤", &[0; 3])], 0), None);
        assert_eq!(SimpleMolkky.next_team(&[team("赤", &[10; 5])], 0), None);
        assert_eq!(SimpleMolkky.next_team(&teams, 0), Some(1));
        assert_eq!(SimpleMolkky.next_team(&teams, 1), Some(0));
    }

    #[test]
    #[should_panic(expected = "チームIDが重複しています")]
    fn 同じidのチームを二重登録できない() {
        let first = team("赤", &[]);
        SimpleMolkky.judge_game(&[first.clone(), first]);
    }
}

mod scoring {
    use rust_molkky::{Molkky, SimpleMolkky, Skittle};

    mod one_skittle {
        use super::{Molkky, SimpleMolkky, Skittle};

        #[test]
        fn 番号5のスキットルを倒すと5点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(5)]).value(), 5);
        }

        #[test]
        fn 番号1のスキットルを倒すと1点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(1)]).value(), 1);
        }

        #[test]
        fn 番号12のスキットルを倒すと12点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(12)]).value(), 12);
        }
    }

    mod multiple_skittles {
        use super::{Molkky, SimpleMolkky, Skittle};

        #[test]
        fn 複数本倒すと倒した本数が点数になる() {
            assert_eq!(
                SimpleMolkky
                    .calc_score(&[Skittle::new(5), Skittle::new(8)])
                    .value(),
                2
            );
        }

        #[test]
        fn 番号1と5と12の3本を倒すと3点になる() {
            assert_eq!(
                SimpleMolkky
                    .calc_score(&[Skittle::new(1), Skittle::new(5), Skittle::new(12)])
                    .value(),
                3
            );
        }

        #[test]
        fn 同じ番号の別スキットルは本数で採点する() {
            assert_eq!(
                SimpleMolkky
                    .calc_score(&[Skittle::new(5), Skittle::new(5)])
                    .value(),
                2
            );
        }

        #[test]
        #[should_panic(expected = "スキットルIDが重複しています")]
        fn 同じ個体のコピーを重複として拒否する() {
            let pin = Skittle::new(5);
            SimpleMolkky.calc_score(&[pin, pin]);
        }
    }

    mod no_skittles {
        use super::{Molkky, SimpleMolkky};

        #[test]
        fn 一本も倒さなければ0点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[]).value(), 0);
        }
    }

    mod immutability {
        use super::{Molkky, SimpleMolkky, Skittle};

        #[test]
        fn 元の配列を変更しても点数は変わらない() {
            let mut skittles = [Skittle::new(5)];
            let thrown = SimpleMolkky.calc_score(&skittles);

            skittles[0] = Skittle::new(12);

            assert_eq!(skittles[0].number(), 12);
            assert_eq!(thrown.value(), 5);
        }
    }

    #[test]
    fn 標準の十二本に別個体の十二番を追加した十三本を採点できる() {
        let mut pins: Vec<_> = (1..=12).map(Skittle::new).collect();
        pins.push(Skittle::new(12));
        assert_eq!(SimpleMolkky.calc_score(&pins).value(), 13);
    }
}

#[test]
fn デフォルト投てきへ棒と個体と乱数源を渡す() {
    use rand::{SeedableRng, rngs::StdRng};
    use rust_molkky::Stick;
    use rust_molkky::{Molkky, SimpleMolkky};
    let rules: &dyn Molkky = &SimpleMolkky;
    let pins = [Skittle::new(12), Skittle::new(12)];
    let fallen = rules.throw(&Stick::new(1.0), &pins, &mut StdRng::seed_from_u64(42));
    assert_eq!(fallen, pins);
    assert_eq!(rules.calc_score(&fallen).value(), 2);
}

mod throwing {
    use crate::support::ConstantRng;
    use rand::{SeedableRng, rngs::StdRng};
    use rust_molkky::simulation::{ArrayField, StandardFieldGenerator};
    use rust_molkky::{Field, FieldGenerator, Molkky, SimpleMolkky, Skittle, Stick};

    #[test]
    fn 確率零なら倒れず確率一なら全ピンが倒れる() {
        let field = StandardFieldGenerator.generate();
        assert!(
            SimpleMolkky
                .throw(&Stick::new(0.0), field.skittles(), &mut ConstantRng(0))
                .is_empty()
        );
        let all = SimpleMolkky.throw(
            &Stick::new(1.0),
            field.skittles(),
            &mut ConstantRng(u64::MAX),
        );
        assert_eq!(all, field.skittles());
        assert_eq!(SimpleMolkky.calc_score(&all).value(), 12);
    }
    #[test]
    fn 投てきは入力のピンと棒の条件を変えない() {
        let pin = Skittle::new(5);
        let field = ArrayField::new(vec![pin]);
        let stick = Stick::new(1.0);
        assert_eq!(
            SimpleMolkky.throw(&stick, field.skittles(), &mut StdRng::seed_from_u64(42)),
            [pin]
        );
        assert_eq!(field.skittles(), [pin]);
        assert_eq!(stick.fall_probability(), 1.0);
    }
    #[test]
    fn 同じピンと確率とシードなら同じ投てきを再現する() {
        let field = StandardFieldGenerator.generate();
        let (mut a, mut b) = (StdRng::seed_from_u64(42), StdRng::seed_from_u64(42));
        for _ in 0..100 {
            assert_eq!(
                SimpleMolkky.throw(&Stick::default(), field.skittles(), &mut a),
                SimpleMolkky.throw(&Stick::default(), field.skittles(), &mut b)
            );
        }
    }
    #[test]
    fn 同じ乱数でも棒の設定を変えると結果が変わる() {
        let pins = [Skittle::new(5)];
        assert!(
            SimpleMolkky
                .throw(&Stick::new(0.2), &pins, &mut ConstantRng(1 << 62))
                .is_empty()
        );
        assert_eq!(
            SimpleMolkky.throw(&Stick::new(0.5), &pins, &mut ConstantRng(1 << 62)),
            pins
        );
    }
    #[test]
    fn 対象が空なら何も倒れない() {
        assert!(
            SimpleMolkky
                .throw(&Stick::default(), &[], &mut ConstantRng(0))
                .is_empty()
        );
    }
    #[test]
    #[should_panic(expected = "スキットルIDが重複しています")]
    fn 同じ個体を重複して投げられない() {
        let pin = Skittle::new(5);
        SimpleMolkky.throw(&Stick::default(), &[pin, pin], &mut ConstantRng(0));
    }
}
