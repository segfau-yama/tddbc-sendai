use rust_molkky::{Molkky, Point, ScoreBoard, ScoreState, SimpleMolkky, Skittle};

fn point(number: i32) -> Point {
    if number == 0 {
        SimpleMolkky.calc_score(&[])
    } else {
        SimpleMolkky.calc_score(&[Skittle::new(number)])
    }
}

fn board(numbers: &[i32]) -> ScoreBoard {
    let mut board = ScoreBoard::default();
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
    assert!(!matches!(
        ScoreBoard::default().state(),
        ScoreState::Disqualified
    ));
    let twice = board(&[5, 0, 0]);
    assert!(!matches!(twice.state(), ScoreState::Disqualified));
    let third = twice.add(point(0));
    assert!(matches!(third.state(), ScoreState::Disqualified));
    assert_eq!(third.total(), 5);
    assert!(!matches!(twice.state(), ScoreState::Disqualified));
    assert!(!matches!(
        board(&[0, 0, 1, 0, 0]).state(),
        ScoreState::Disqualified
    ));
}

#[test]
fn 採点側で直近の超過補正と勝利を判定する() {
    let original = board(&[12, 12, 12, 12, 1]);
    assert!(!matches!(original.state(), ScoreState::Reset));
    assert!(!matches!(original.state(), ScoreState::Won));
    assert!(matches!(original.add(point(1)).state(), ScoreState::Won));
    let reset = original.add(point(2));
    assert!(matches!(reset.state(), ScoreState::Reset));
    assert_eq!(reset.total(), 25);
    let next = reset.add(point(12));
    assert!(!matches!(next.state(), ScoreState::Reset));
    assert_eq!(next.total(), 37);
}

#[test]
fn 一回も投てきしていなければ0点になる() {
    let board = ScoreBoard::default();
    assert_eq!(board.total(), 0);
    assert!(board.history().is_empty());
}

#[test]
fn 一回目に5本倒すと5点になる() {
    let mut board = ScoreBoard::default();
    board = board.add(SimpleMolkky.calc_score(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]));
    assert_eq!(board.total(), 5);
}

#[test]
fn 五本倒した後に11番を1本倒すと16点になる() {
    let mut board = ScoreBoard::default();
    board = board.add(SimpleMolkky.calc_score(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]));
    board = board.add(point(11));
    assert_eq!(board.total(), 16);
}

#[test]
fn 零点の投てきも履歴に残り現在の得点を変えない() {
    let board = board(&[11, 0]);
    assert_eq!(board.total(), 11);
    assert_eq!(history(&board), [11, 0]);
}

#[test]
fn 同じ点数を2回追加しても2回分合計する() {
    let mut board = ScoreBoard::default();
    let eleven = point(11);
    board = board.add(eleven);
    board = board.add(eleven);
    assert_eq!(board.total(), 22);
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
    assert_eq!(original.total(), 5);
    assert_eq!(history(&original), [5]);
    assert_eq!(updated.total(), 16);
    assert_eq!(history(&updated), [5, 11]);
}

mod target_score {
    use super::*;

    #[test]
    fn 四十九点はそのまま保持する() {
        assert_eq!(board(&[12, 12, 12, 12, 1]).total(), 49);
    }

    #[test]
    fn 五十点ちょうどはそのまま保持する() {
        let mut board = board(&[12, 12, 12, 12, 1]);
        board = board.add(point(1));
        assert_eq!(board.total(), 50);
    }

    #[test]
    fn 五十点を超えた投てきで25点に戻る() {
        let original = board(&[12, 12, 12, 12, 1]);
        let updated = original.add(point(2));
        assert_eq!(updated.total(), 25);
        assert_eq!(history(&updated), [25]);
        assert_eq!(history(&original), [12, 12, 12, 12, 1]);
        assert_eq!(original.total(), 49);
    }

    #[test]
    fn 超過後の点数は25点から加算し再び超過しても25点に戻る() {
        let mut updated = board(&[12, 12, 12, 12, 1, 2]);
        updated = updated.add(point(12));
        assert_eq!(updated.total(), 37);
        assert_eq!(history(&updated), [25, 12]);
        updated = updated.add(point(12));
        assert_eq!(updated.total(), 49);
        updated = updated.add(point(2));
        assert_eq!(updated.total(), 25);
        assert_eq!(history(&updated), [25]);
        let replayed = ScoreBoard::new([12, 12, 12, 12, 1, 2, 12, 12, 2].map(point).to_vec());
        assert_eq!(replayed.total(), updated.total());
        assert_eq!(replayed.history(), updated.history());
        assert_eq!(replayed.state(), ScoreState::Reset);
    }
}

mod immutability {
    use super::*;

    #[test]
    fn 点数を追加しても元の得点表の履歴と得点は変わらない() {
        let five = SimpleMolkky.calc_score(&[Skittle::new(5)]);
        let eleven = SimpleMolkky.calc_score(&[Skittle::new(11)]);
        let original = ScoreBoard::default().add(five);
        let updated = original.add(eleven);
        assert_eq!(original.history(), &[five]);
        assert_eq!(original.total(), 5);
        assert_eq!(updated.history(), &[five, eleven]);
        assert_eq!(updated.total(), 16);
    }
}

mod states {
    use super::*;
    use rust_molkky::ScoreState;

    #[test]
    fn 履歴復元で状態と補正済み合計を確定する() {
        for (points, total, state, expected_history) in [
            (&[][..], 0, ScoreState::Calculated, &[][..]),
            (
                &[12, 12, 12, 12, 1][..],
                49,
                ScoreState::Calculated,
                &[12, 12, 12, 12, 1][..],
            ),
            (
                &[12, 12, 12, 12, 1, 1][..],
                50,
                ScoreState::Won,
                &[12, 12, 12, 12, 1, 1][..],
            ),
            (&[12, 12, 12, 12, 3][..], 25, ScoreState::Reset, &[25][..]),
            (
                &[12, 12, 12, 12, 3, 1][..],
                26,
                ScoreState::Calculated,
                &[25, 1][..],
            ),
            (
                &[5, 0, 0, 0][..],
                5,
                ScoreState::Disqualified,
                &[5, 0, 0, 0][..],
            ),
            (
                &[0, 0, 1, 0, 0][..],
                1,
                ScoreState::Calculated,
                &[0, 0, 1, 0, 0][..],
            ),
        ] {
            let board = ScoreBoard::new(points.iter().map(|&n| point(n)).collect());
            assert_eq!(history(&board), expected_history);
            assert_eq!(board.total(), total);
            assert_eq!(board.state(), state);
        }
    }

    #[test]
    fn 勝利と失格の後の履歴は生成と追加の両方で拒否する() {
        for points in [&[10, 10, 10, 10, 10][..], &[5, 0, 0, 0][..]] {
            let history: Vec<_> = points.iter().map(|&n| point(n)).collect();
            let board = ScoreBoard::new(history.clone());
            for next in [point(0), point(1)] {
                let mut invalid = history.clone();
                invalid.push(next);
                assert!(std::panic::catch_unwind(|| ScoreBoard::new(invalid)).is_err());
                assert!(std::panic::catch_unwind(|| board.add(next)).is_err());
                assert_eq!(board.history(), history);
            }
        }
    }
}

#[test]
fn リセット後の零点で通常状態へ戻しても元の状態は変わらない() {
    let original = ScoreBoard::new([12, 12, 12, 12, 3].map(point).to_vec());
    let updated = original.add(point(0));
    assert_eq!(original.state(), ScoreState::Reset);
    assert_eq!(original.clone().state(), ScoreState::Reset);
    assert_eq!(updated.state(), ScoreState::Calculated);
    assert_eq!(original.total(), 25);
    assert_eq!(updated.total(), 25);
    assert_eq!(history(&original), [25]);
    assert_eq!(history(&updated), [25, 0]);
    assert_eq!(ScoreBoard::default().state(), ScoreState::Calculated);
}

#[test]
fn 超過は連続零点を中断し補正後の三連続零点で失格になる() {
    let original = board(&[12, 12, 12, 12, 1, 0, 0]);
    let reset = original.add(point(2));
    assert_eq!(history(&reset), [25]);
    let twice = reset.add(point(0)).add(point(0));
    assert_eq!(twice.state(), ScoreState::Calculated);
    let third = twice.add(point(0));
    assert_eq!(third.state(), ScoreState::Disqualified);
    assert_eq!(third.total(), 25);
    assert_eq!(history(&third), [25, 0, 0, 0]);
    let raw = [12, 12, 12, 12, 1, 0, 0, 2, 0, 0, 0].map(point).to_vec();
    let restored = ScoreBoard::new(raw.clone());
    assert_eq!(restored.history(), third.history());
    assert_eq!(restored.state(), third.state());
    let mut invalid = raw;
    invalid.push(point(1));
    assert!(std::panic::catch_unwind(|| ScoreBoard::new(invalid)).is_err());
    assert!(std::panic::catch_unwind(|| third.add(point(0))).is_err());
    assert_eq!(original.total(), 49);
}

#[test]
fn 補正後も五十点ちょうどで勝利し以後の記録は拒否する() {
    let raw = [12, 12, 12, 12, 3, 12, 12, 1].map(point).to_vec();
    let restored = ScoreBoard::new(raw.clone());
    let added = board(&[12, 12, 12, 12, 3, 12, 12, 1]);
    assert_eq!(restored.total(), 50);
    assert_eq!(restored.state(), ScoreState::Won);
    assert_eq!(history(&restored), [25, 12, 12, 1]);
    assert_eq!(restored.history(), added.history());
    let mut invalid = raw;
    invalid.push(point(0));
    assert!(std::panic::catch_unwind(|| ScoreBoard::new(invalid)).is_err());
    assert!(std::panic::catch_unwind(|| restored.add(point(1))).is_err());
}
