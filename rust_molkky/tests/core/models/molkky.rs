use rust_molkky::{Molkky, Skittle, Team, calc_score};

#[test]
fn 複数チームを保持し開始時は進行中になる() {
    let game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    assert_eq!(game.teams().len(), 2);
    assert_eq!(game.teams()[0].name(), "赤");
    assert_eq!(game.teams()[1].name(), "青");
    assert_eq!(game.result(), None);
}

#[test]
fn 同名のチームでも指定したチームだけに記録する() {
    let mut game = Molkky::new(vec![Team::new("同じ名前"), Team::new("同じ名前")]);
    let point = calc_score(&[Skittle::new(5)]);
    assert_eq!(point.value(), 5);
    game = game.record_point(1, point);
    assert_eq!(game.teams()[0].score_board().value(), 0);
    assert!(game.teams()[0].score_board().history().is_empty());
    assert_eq!(game.teams()[1].score_board().value(), 5);
    assert_eq!(
        game.teams()[1]
            .score_board()
            .history()
            .iter()
            .map(|point| point.value())
            .collect::<Vec<_>>(),
        &[5]
    );
}

fn game_at_forty_nine() -> Molkky {
    let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    let twelve = calc_score(&[Skittle::new(12)]);
    for _ in 0..4 {
        game = game.record_point(1, twelve);
    }
    game = game.record_point(1, calc_score(&[Skittle::new(1)]));
    game
}

mod invalid_operations {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn 五十点到達後に追加記録がある履歴では構築できない() {
        let ten = calc_score(&[Skittle::new(10)]);
        let mut board = rust_molkky::ScoreBoard::new();
        for _ in 0..5 {
            board = board.add(ten);
        }
        for point in [calc_score(&[]), calc_score(&[Skittle::new(1)])] {
            let invalid = Team::new("赤").with_score_board(board.add(point));
            assert!(catch_unwind(|| Molkky::new(vec![invalid])).is_err());
        }
        assert_eq!(board.value(), 50);
        assert_eq!(board.history().len(), 5);
    }

    #[test]
    #[should_panic(expected = "チームは1件以上必要です")]
    fn チームが空のゲームは作れない() {
        Molkky::new(vec![]);
    }

    #[test]
    #[should_panic(expected = "勝者が複数います")]
    fn 勝者が複数いる状態ではゲームを作れない() {
        let mut red = Team::new("赤");
        let mut blue = Team::new("青");
        let ten = calc_score(&[Skittle::new(10)]);
        for _ in 0..5 {
            red = red.with_score_board(red.score_board().add(ten));
            blue = blue.with_score_board(blue.score_board().add(ten));
        }
        Molkky::new(vec![red, blue]);
    }

    #[test]
    fn 存在しないチームへの記録を拒否して得点を保つ() {
        let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
        game = game.record_point(0, calc_score(&[Skittle::new(5)]));
        for index in [2, usize::MAX] {
            let failed = catch_unwind(AssertUnwindSafe(|| {
                game = game.record_point(index, calc_score(&[Skittle::new(12)]));
            }));
            assert!(failed.is_err());
            assert_eq!(
                game.teams()[0]
                    .score_board()
                    .history()
                    .iter()
                    .map(|point| point.value())
                    .collect::<Vec<_>>(),
                &[5]
            );
            assert!(game.teams()[1].score_board().history().is_empty());
            assert_eq!(game.result(), None);
        }
    }

    #[test]
    fn 終了後はどのチームへの記録も拒否して履歴と勝者を保つ() {
        let mut game = game_at_forty_nine();
        game = game.record_point(1, calc_score(&[Skittle::new(1)]));
        for index in [0, 1] {
            let failed = catch_unwind(AssertUnwindSafe(|| {
                game = game.record_point(index, calc_score(&[Skittle::new(12)]));
            }));
            assert!(failed.is_err());
            assert!(game.teams()[0].score_board().history().is_empty());
            assert_eq!(
                game.teams()[1]
                    .score_board()
                    .history()
                    .iter()
                    .map(|point| point.value())
                    .collect::<Vec<_>>(),
                &[12, 12, 12, 12, 1, 1]
            );
            assert_eq!(game.teams()[1].score_board().value(), 50);
            assert_eq!(game.result(), Some(1));
        }
    }
}

mod target_score {
    use super::*;

    #[test]
    fn 四十九点ではゲームは終了しない() {
        let game = game_at_forty_nine();
        assert_eq!(game.teams()[1].score_board().value(), 49);
        assert_eq!(game.result(), None);
    }

    #[test]
    fn 五十点ちょうどで該当チームの勝利になる() {
        let mut game = game_at_forty_nine();
        game = game.record_point(1, calc_score(&[Skittle::new(1)]));
        assert_eq!(game.teams()[1].score_board().value(), 50);
        assert_eq!(game.result(), Some(1));
    }

    #[test]
    fn 超過時は25点に戻してゲームを継続する() {
        let mut game = game_at_forty_nine();
        let point = calc_score(&[Skittle::new(2)]);
        assert_eq!(point.value(), 2);
        game = game.record_point(1, point);
        assert_eq!(game.teams()[1].score_board().value(), 25);
        assert_eq!(game.result(), None);
        game = game.record_point(1, calc_score(&[Skittle::new(12)]));
        assert_eq!(game.teams()[1].score_board().value(), 37);
        game = game.record_point(0, calc_score(&[Skittle::new(5)]));
        assert_eq!(game.teams()[0].score_board().value(), 5);
        game = game.record_point(1, calc_score(&[Skittle::new(12)]));
        game = game.record_point(1, calc_score(&[Skittle::new(1)]));
        assert_eq!(game.teams()[1].score_board().value(), 50);
        assert_eq!(game.result(), Some(1));
    }
}

#[test]
fn 一チームのゲームと既に勝者がいるゲームも表現できる() {
    assert_eq!(Molkky::new(vec![Team::new("赤")]).result(), None);
    let mut winner = Team::new("赤");
    let ten = calc_score(&[Skittle::new(10)]);
    for _ in 0..5 {
        winner = winner.with_score_board(winner.score_board().add(ten));
    }
    assert_eq!(Molkky::new(vec![winner]).result(), Some(0));
}

mod restored_rules {
    use super::*;
    use rust_molkky::ScoreBoard;
    use std::panic::catch_unwind;

    fn team_with_points(name: &str, points: &[i32]) -> Team {
        let mut board = ScoreBoard::new();
        for &number in points {
            let point = if number == 0 {
                calc_score(&[])
            } else {
                calc_score(&[Skittle::new(number)])
            };
            board = board.add(point);
        }
        Team::new(name).with_score_board(board)
    }

    #[test]
    fn 複数チームから残り一チームになったら勝利する() {
        let game = Molkky::new(vec![team_with_points("赤", &[0, 0, 0]), Team::new("青")]);
        assert_eq!(game.result(), Some(1));
        assert!(game.is_finished());
        assert_eq!(game.teams()[1].score_board().value(), 0);
        assert!(catch_unwind(|| game.record_point(1, calc_score(&[Skittle::new(1)]))).is_err());
    }

    #[test]
    fn 記録で残り一チームになった時点で終了し元のゲームは変わらない() {
        let game = Molkky::new(vec![team_with_points("赤", &[0, 0]), Team::new("青")]);
        assert!(!game.is_finished());
        let finished = game.record_point(0, calc_score(&[]));
        assert!(finished.is_finished());
        assert_eq!(finished.result(), Some(1));
        assert_eq!(finished.teams()[1].score_board().value(), 0);
        assert!(!game.is_finished());
        assert_eq!(game.result(), None);
        assert_eq!(game.teams()[0].score_board().history().len(), 2);
        assert!(catch_unwind(|| finished.record_point(1, calc_score(&[Skittle::new(1)]))).is_err());
    }

    #[test]
    fn 超過した履歴も25点補正後の得点で構築できる() {
        let game = Molkky::new(vec![team_with_points("赤", &[12, 12, 12, 12, 3])]);
        assert_eq!(game.teams()[0].score_board().value(), 25);
        assert_eq!(game.result(), None);
        assert!(!game.is_finished());
    }

    #[test]
    fn 失格チームがいても二チーム以上残っていれば継続する() {
        let game = Molkky::new(vec![
            team_with_points("赤", &[0, 0, 0]),
            Team::new("青"),
            Team::new("緑"),
        ]);
        assert!(game.teams()[0].score_board().is_disqualified());
        assert!(!game.is_finished());
        assert_eq!(game.result(), None);
        let updated = game.record_point(1, calc_score(&[Skittle::new(5)]));
        assert_eq!(updated.teams()[1].score_board().value(), 5);
        assert_eq!(game.teams()[1].score_board().value(), 0);
    }

    #[test]
    fn 失格後に追加記録のある履歴は構築できない() {
        let disqualified = team_with_points("赤", &[5, 0, 0, 0]);
        for point in [calc_score(&[]), calc_score(&[Skittle::new(1)])] {
            let invalid = disqualified.with_score_board(disqualified.score_board().add(point));
            assert!(catch_unwind(|| Molkky::new(vec![invalid])).is_err());
        }
        assert_eq!(disqualified.score_board().value(), 5);
        assert_eq!(disqualified.score_board().history().len(), 4);
    }

    #[test]
    fn 三回目の0点で失格にし失格チームへの記録だけを拒否する() {
        let game = Molkky::new(vec![
            team_with_points("赤", &[5, 0, 0]),
            Team::new("青"),
            Team::new("緑"),
        ]);
        let disqualified = game.record_point(0, calc_score(&[]));
        assert!(disqualified.teams()[0].score_board().is_disqualified());
        assert!(!disqualified.is_finished());
        assert!(
            catch_unwind(|| disqualified.record_point(0, calc_score(&[Skittle::new(1)]))).is_err()
        );
        let continued = disqualified.record_point(1, calc_score(&[Skittle::new(11)]));
        assert_eq!(continued.teams()[1].score_board().value(), 11);
        assert_eq!(
            continued.teams()[0].score_board().history(),
            disqualified.teams()[0].score_board().history()
        );
        assert!(!game.teams()[0].score_board().is_disqualified());
        assert_eq!(game.teams()[0].score_board().history().len(), 3);
    }

    #[test]
    fn 全チーム失格なら勝者なしで終了して記録を拒否する() {
        let game = Molkky::new(vec![
            team_with_points("赤", &[0, 0, 0]),
            team_with_points("青", &[1, 0, 0, 0]),
        ]);
        assert!(game.is_finished());
        assert_eq!(game.result(), None);
        for index in [0, 1] {
            assert!(catch_unwind(|| game.record_point(index, calc_score(&[]))).is_err());
        }
        assert_eq!(game.teams()[0].score_board().value(), 0);
        assert_eq!(game.teams()[1].score_board().value(), 1);
    }

    #[test]
    fn 正の点数で連続0点の回数がリセットされる() {
        let game = Molkky::new(vec![team_with_points("赤", &[0, 0, 1, 0, 0])]);
        assert!(!game.teams()[0].score_board().is_disqualified());
        assert!(!game.is_finished());
        let updated = game.record_point(0, calc_score(&[]));
        assert!(updated.teams()[0].score_board().is_disqualified());
        assert!(updated.is_finished());
    }

    #[test]
    fn 二チーム以上残っていても50点到達で勝利する() {
        let game = Molkky::new(vec![
            team_with_points("赤", &[0, 0, 0]),
            team_with_points("青", &[12, 12, 12, 12, 1]),
            Team::new("緑"),
        ]);
        assert!(!game.is_finished());
        let won = game.record_point(1, calc_score(&[Skittle::new(1)]));
        assert!(won.is_finished());
        assert_eq!(won.result(), Some(1));
        assert_eq!(game.result(), None);
        assert_eq!(game.teams()[1].score_board().value(), 49);
    }
}

mod immutability {
    use super::*;
    use rust_molkky::{Field, GameStatus, Stick};
    #[test]
    fn 共通のピン集合と棒の設定を不変に差し替える() {
        let original = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
        let configured = original
            .with_field(Field::new(vec![Skittle::new(5)]))
            .with_stick(Stick::new(0.75));
        let scored = configured.record_point(0, calc_score(&[Skittle::new(5)]));
        assert_eq!(original.field().skittles().len(), 12);
        assert_eq!(original.stick().fall_probability(), 0.25);
        assert_eq!(configured.field().skittles(), [Skittle::new(5)]);
        assert_eq!(scored.field().skittles(), configured.field().skittles());
        assert_eq!(scored.stick(), configured.stick());
        assert_eq!(configured.teams()[0].score_board().value(), 0);
        assert_eq!(scored.teams()[0].score_board().value(), 5);
        assert_eq!(scored.status(), GameStatus::InProgress);
    }
    #[test]
    fn ゲームへの記録でも元のゲームと他チームは変わらない() {
        let original = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
        let updated = original.record_point(0, calc_score(&[Skittle::new(11)]));
        assert_eq!(original.teams()[0].score_board().value(), 0);
        assert_eq!(original.teams()[1].score_board().value(), 0);
        assert_eq!(updated.teams()[0].score_board().value(), 11);
        assert_eq!(updated.teams()[1].score_board().value(), 0);
    }
}
