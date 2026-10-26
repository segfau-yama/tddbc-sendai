use rust_molkky::{Point, ScoreBoard, Skittle, calc_score};

fn point(number: i32) -> Point {
    if number == 0 {
        calc_score(&[])
    } else {
        calc_score(&[Skittle::new(number)])
    }
}

fn board(numbers: &[i32]) -> ScoreBoard {
    let mut board = ScoreBoard::new();
    for &number in numbers {
        board = board.add(point(number));
    }
    board
}

fn history(board: &ScoreBoard) -> Vec<i32> {
    board.history().iter().map(Point::value).collect()
}

#[test]
fn 三回連続0点のチームを失格と判定する() {
    assert!(!ScoreBoard::new().is_disqualified());
    let twice = board(&[5, 0, 0]);
    assert!(!twice.is_disqualified());
    let third = twice.add(point(0));
    assert!(third.is_disqualified());
    assert_eq!(third.value(), 5);
    assert!(!twice.is_disqualified());
    assert!(!board(&[0, 0, 1, 0, 0]).is_disqualified());
}

#[test]
fn 採点側で直近の超過補正と勝利を判定する() {
    let original = board(&[12, 12, 12, 12, 1]);
    assert!(!original.was_reset());
    assert!(!original.is_winner());
    assert!(original.add(point(1)).is_winner());
    let reset = original.add(point(2));
    assert!(reset.was_reset());
    assert_eq!(reset.value(), 25);
    let next = reset.add(point(12));
    assert!(!next.was_reset());
    assert_eq!(next.value(), 37);
}

#[test]
fn 一回も投てきしていなければ0点になる() {
    let board = ScoreBoard::new();
    assert_eq!(board.value(), 0);
    assert!(board.history().is_empty());
}

#[test]
fn 一回目に5本倒すと5点になる() {
    let mut board = ScoreBoard::new();
    board = board.add(calc_score(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]));
    assert_eq!(board.value(), 5);
}

#[test]
fn 五本倒した後に11番を1本倒すと16点になる() {
    let mut board = ScoreBoard::new();
    board = board.add(calc_score(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]));
    board = board.add(point(11));
    assert_eq!(board.value(), 16);
}

#[test]
fn 零点の投てきも履歴に残り現在の得点を変えない() {
    let board = board(&[11, 0]);
    assert_eq!(board.value(), 11);
    assert_eq!(history(&board), [11, 0]);
}

#[test]
fn 同じ点数を2回追加しても2回分合計する() {
    let mut board = ScoreBoard::new();
    let eleven = point(11);
    board = board.add(eleven);
    board = board.add(eleven);
    assert_eq!(board.value(), 22);
    assert_eq!(history(&board), [11, 11]);
}

#[test]
fn 投てきの点数を順番どおり履歴に保持する() {
    assert_eq!(history(&board(&[5, 11])), [5, 11]);
}

#[test]
fn 得点表に追加しても別の得点表は変わらない() {
    let original = board(&[5]);
    let mut updated = board(&[5]);
    updated = updated.add(point(11));
    assert_eq!(original.value(), 5);
    assert_eq!(history(&original), [5]);
    assert_eq!(updated.value(), 16);
    assert_eq!(history(&updated), [5, 11]);
}

mod target_score {
    use super::*;

    #[test]
    fn 四十九点はそのまま保持する() {
        assert_eq!(board(&[12, 12, 12, 12, 1]).value(), 49);
    }

    #[test]
    fn 五十点ちょうどはそのまま保持する() {
        let mut board = board(&[12, 12, 12, 12, 1]);
        board = board.add(point(1));
        assert_eq!(board.value(), 50);
    }

    #[test]
    fn 五十点を超えた投てきで25点に戻る() {
        let mut board = board(&[12, 12, 12, 12, 1]);
        board = board.add(point(2));
        assert_eq!(board.value(), 25);
        assert_eq!(history(&board), [12, 12, 12, 12, 1, 2]);
    }

    #[test]
    fn 超過後の点数は25点から加算し再び超過しても25点に戻る() {
        let mut updated = board(&[12, 12, 12, 12, 1, 2]);
        updated = updated.add(point(12));
        assert_eq!(updated.value(), 37);
        updated = updated.add(point(12));
        assert_eq!(updated.value(), 49);
        updated = updated.add(point(2));
        assert_eq!(updated.value(), 25);
        let replayed = board(&[12, 12, 12, 12, 1, 2, 12, 12, 2]);
        assert_eq!(replayed.value(), updated.value());
        assert_eq!(replayed.history(), updated.history());
    }
}

mod scoring {
    use rust_molkky::{Skittle, calc_score};

    mod one_skittle {
        use super::{Skittle, calc_score};

        #[test]
        fn 番号5のスキットルを倒すと5点になる() {
            assert_eq!(calc_score(&[Skittle::new(5)]).value(), 5);
        }

        #[test]
        fn 番号1のスキットルを倒すと1点になる() {
            assert_eq!(calc_score(&[Skittle::new(1)]).value(), 1);
        }

        #[test]
        fn 番号12のスキットルを倒すと12点になる() {
            assert_eq!(calc_score(&[Skittle::new(12)]).value(), 12);
        }
    }

    mod multiple_skittles {
        use super::{Skittle, calc_score};

        #[test]
        fn 複数本倒すと倒した本数が点数になる() {
            assert_eq!(calc_score(&[Skittle::new(5), Skittle::new(8)]).value(), 2);
        }

        #[test]
        fn 番号1と5と12の3本を倒すと3点になる() {
            assert_eq!(
                calc_score(&[Skittle::new(1), Skittle::new(5), Skittle::new(12)]).value(),
                3
            );
        }

        #[test]
        #[should_panic(expected = "スキットル番号が重複しています: 5")]
        fn 同じ番号の別スキットルを重複として拒否する() {
            calc_score(&[Skittle::new(5), Skittle::new(5)]);
        }

        #[test]
        #[should_panic(expected = "スキットル番号が重複しています: 5")]
        fn 同じ番号は位置が違っても重複として拒否する() {
            use rust_molkky::Position;
            calc_score(&[
                Skittle::new(5),
                Skittle::new(5).with_position(Position::new(3, 4)),
            ]);
        }
    }

    mod no_skittles {
        use super::calc_score;

        #[test]
        fn 一本も倒さなければ0点になる() {
            assert_eq!(calc_score(&[]).value(), 0);
        }
    }

    mod immutability {
        use super::{Skittle, calc_score};

        #[test]
        fn 元の配列を変更しても点数は変わらない() {
            let mut skittles = [Skittle::new(5)];
            let thrown = calc_score(&skittles);

            skittles[0] = Skittle::new(12);

            assert_eq!(skittles[0].number(), 12);
            assert_eq!(thrown.value(), 5);
        }
    }
}
