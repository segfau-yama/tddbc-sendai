use rust_molkky::{Point, ScoreBoard, ScoreState};

fn board(points: &[i32]) -> ScoreBoard {
    points.iter().fold(ScoreBoard::default(), |board, &value| {
        board.add(Point::new(value))
    })
}

// 得点計算をテスト側へ複製せず、保存された履歴を実際のモデルで再生する。
fn score_history(board: &ScoreBoard) -> Vec<i32> {
    let history = board.history();
    (1..=history.len())
        .map(|end| ScoreBoard::new(history[..end].to_vec()).total())
        .collect()
}

fn assert_recording_rejected(board: &ScoreBoard, value: i32) {
    let history = board.history();
    let total = board.total();
    let state = board.state();
    assert!(std::panic::catch_unwind(|| board.add(Point::new(value))).is_err());
    assert_eq!(board.history(), history);
    assert_eq!(board.total(), total);
    assert_eq!(board.state(), state);
}

mod recording {
    use super::*;

    mod empty {
        use super::*;

        #[test]
        fn s02_1_投てき前の得点は零点になる() {
            let board = ScoreBoard::default();
            assert_eq!(board.total(), 0);
            assert!(board.history().is_empty());
        }
    }

    mod addition {
        use super::*;

        #[test]
        fn s02_2_五点の後に十一点を記録すると十六点になる() {
            assert_eq!(board(&[5, 11]).total(), 16);
        }

        #[test]
        fn s02_2_同じ点数の投てきもそれぞれ加算する() {
            assert_eq!(board(&[11, 11]).total(), 22);
        }
    }

    mod zero_points {
        use super::*;

        #[test]
        fn s02_3_零点も一投として残り得点は変わらない() {
            let board = board(&[11, 0]);
            assert_eq!(board.total(), 11);
            assert_eq!(board.history().len(), 2);
        }

        #[test]
        fn s02_3_零点の後にも同じ番号の点数を記録できる() {
            let board = board(&[11, 0, 11]);
            assert_eq!(board.total(), 22);
            assert_eq!(board.history().len(), 3);
        }
    }
}

mod score_history {
    use super::*;

    #[test]
    fn s03_1_各投てき終了時の得点を保存履歴から確認できる() {
        assert_eq!(score_history(&board(&[5, 11, 0, 3])), [5, 16, 16, 19]);
    }

    #[test]
    fn s03_2_記録がなければ得点推移は空になる() {
        assert!(score_history(&ScoreBoard::default()).is_empty());
    }

    #[test]
    fn s03_2_過去の得点を参照しても元の記録を変更しない() {
        let board = board(&[5, 11]);
        let history = board.history();
        assert_eq!(score_history(&board), [5, 16]);
        assert_eq!(board.total(), 16);
        assert_eq!(board.history(), history);
    }
}

mod target_score {
    use super::*;

    mod reaching_target {
        use super::*;

        #[test]
        fn s04_1_四十九点なら投てきを続けられる() {
            let board = board(&[12, 12, 12, 12, 1]);
            assert_eq!(board.total(), 49);
            assert_ne!(board.state(), ScoreState::Won);
            assert_eq!(board.add(Point::new(0)).total(), 49);
        }

        #[test]
        fn s04_1_四十八点から二点で五十点到達を判定できる() {
            let board = board(&[12, 12, 12, 12]).add(Point::new(2));
            assert_eq!(board.total(), 50);
            assert_eq!(board.state(), ScoreState::Won);
        }

        #[test]
        fn 補正後も五十点到達を逐次追加と一括復元で判定できる() {
            let points = [12, 12, 12, 12, 3, 12, 12, 1];
            let added = board(&points);
            assert_eq!(added.total(), 50);
            assert_eq!(added.state(), ScoreState::Won);
            let restored = ScoreBoard::new(points.map(Point::new).to_vec());
            assert_eq!(restored.total(), 50);
            assert_eq!(restored.state(), ScoreState::Won);
            assert_eq!(score_history(&restored), [12, 24, 36, 48, 25, 37, 49, 50]);
            assert_recording_rejected(&restored, 1);
            let mut invalid = points.map(Point::new).to_vec();
            invalid.push(Point::new(0));
            assert!(std::panic::catch_unwind(|| ScoreBoard::new(invalid)).is_err());
        }
    }

    mod exceeding_target {
        use super::*;

        #[test]
        fn s04_2_補正前後すべての得点推移を確認できる() {
            let board = board(&[12, 12, 12, 12, 3, 10]);
            assert_eq!(board.total(), 35);
            assert_eq!(score_history(&board), [12, 24, 36, 48, 25, 35]);
        }

        #[test]
        fn s04_2_一括で履歴を復元しても補正後から加算する() {
            let history = [12, 12, 12, 12, 3, 10].map(Point::new).to_vec();
            assert_eq!(ScoreBoard::new(history).total(), 35);
        }

        #[test]
        fn s04_3_超過後も加算でき再び超過したら二十五点に戻る() {
            let reset = board(&[12, 12, 12, 12, 1]).add(Point::new(2));
            assert_eq!(reset.total(), 25);
            let next = reset.add(Point::new(12));
            assert_eq!(next.total(), 37);
            let next = next.add(Point::new(12));
            assert_eq!(next.total(), 49);
            assert_eq!(next.add(Point::new(2)).total(), 25);
        }
    }

    mod recording_after_end {
        use super::*;

        #[test]
        fn s04_4_五十点の後は正の点数を追加できない() {
            assert_recording_rejected(&board(&[12, 12, 12, 12, 2]), 1);
        }

        #[test]
        fn s04_4_五十点の後は零点も追加できない() {
            assert_recording_rejected(&board(&[12, 12, 12, 12, 2]), 0);
        }

        #[test]
        fn s04_4_一括の履歴も五十点到達後の記録を拒否する() {
            for next in [0, 1] {
                let history = [12, 12, 12, 12, 2, next].map(Point::new).to_vec();
                assert!(std::panic::catch_unwind(|| ScoreBoard::new(history)).is_err());
            }
        }
    }
}

mod disqualification {
    use super::*;

    mod third_miss {
        use super::*;

        #[test]
        fn 通常零点が二回までなら失格にしない() {
            let board = board(&[12, 0, 0]);
            assert_eq!(board.total(), 12);
            assert_ne!(board.state(), ScoreState::Disqualified);
        }

        #[test]
        fn 三回連続の通常零点で失格となり得点は零点になる() {
            let board = board(&[12, 0, 0, 0]);
            assert_eq!(board.state(), ScoreState::Disqualified);
            assert_eq!(board.total(), 0);
        }

        #[test]
        fn 補正後も三回連続の通常零点で失格となり得点は零点になる() {
            let reset = board(&[12, 12, 12, 12, 1, 0, 0, 2]);
            assert_eq!(reset.state(), ScoreState::Reset);
            let twice = reset.add(Point::new(0)).add(Point::new(0));
            assert_ne!(twice.state(), ScoreState::Disqualified);
            let third = twice.add(Point::new(0));
            assert_eq!(third.state(), ScoreState::Disqualified);
            assert_eq!(third.total(), 0);
        }

        #[test]
        fn 補正を含む一括履歴でも三回連続の通常零点で失格になる() {
            let history = [12, 12, 12, 12, 1, 0, 0, 2, 0, 0, 0]
                .map(Point::new)
                .to_vec();
            let board = ScoreBoard::new(history);
            assert_eq!(board.state(), ScoreState::Disqualified);
            assert_eq!(board.total(), 0);
        }

        #[test]
        fn s08_2_正の点数で中断された零点は合計三回以上でも失格にしない() {
            let board = board(&[0, 0, 1, 0, 0]);
            assert_eq!(board.total(), 1);
            assert_ne!(board.state(), ScoreState::Disqualified);
        }
    }

    mod recording_after_end {
        use super::*;

        #[test]
        fn 失格後は正の点数を追加できない() {
            assert_recording_rejected(&board(&[12, 0, 0, 0]), 1);
        }

        #[test]
        fn 失格後は零点も追加できない() {
            assert_recording_rejected(&board(&[12, 0, 0, 0]), 0);
        }

        #[test]
        fn 一括の履歴も失格後の記録を拒否する() {
            for next in [0, 1] {
                let history = [12, 0, 0, 0, next].map(Point::new).to_vec();
                assert!(std::panic::catch_unwind(|| ScoreBoard::new(history)).is_err());
            }
        }
    }
}

mod immutability {
    use super::*;

    #[test]
    fn 得点を追加しても元の履歴と得点は変わらない() {
        let original = board(&[5]);
        let history = original.history();
        let updated = original.add(Point::new(11));
        assert_eq!(original.total(), 5);
        assert_eq!(original.history(), history);
        assert_eq!(updated.total(), 16);
    }

    #[test]
    fn 補正後に投てきを続けても元の状態は変わらない() {
        let original = board(&[12, 12, 12, 12, 3]);
        let updated = original.add(Point::new(0));
        assert_eq!(original.state(), ScoreState::Reset);
        assert_eq!(updated.state(), ScoreState::Calculated);
        assert_eq!(original.total(), 25);
        assert_eq!(updated.total(), 25);
    }
}

mod forfeit_victory {
    use super::*;

    #[test]
    fn 他チームの失格で勝利しても投てき履歴を追加しない() {
        let original = board(&[5, 8]);
        let winner = original.win_by_default();
        assert_eq!(original.total(), 13);
        assert_eq!(winner.total(), 50);
        assert_eq!(winner.state(), ScoreState::Won);
        assert_eq!(winner.history(), original.history());
        assert_recording_rejected(&winner, 0);
    }
}
