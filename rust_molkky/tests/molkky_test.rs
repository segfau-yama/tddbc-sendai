use rust_molkky::{Molkky, Skittle, Team, Throw};

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
    assert_eq!(game.record_throw(1, &Throw::new(&[Skittle::new(5)])), 5);
    assert_eq!(game.teams()[0].score().value(), 0);
    assert!(game.teams()[0].score().history().is_empty());
    assert_eq!(game.teams()[1].score().value(), 5);
    assert_eq!(game.teams()[1].score().history(), &[5]);
}

fn game_at_forty_nine() -> Molkky {
    let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    let twelve = Throw::new(&[Skittle::new(12)]);
    for _ in 0..4 {
        game.record_throw(1, &twelve);
    }
    game.record_throw(1, &Throw::new(&[Skittle::new(1)]));
    game
}

mod invalid_operations {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

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
        let ten = Throw::new(&[Skittle::new(10)]);
        for _ in 0..5 {
            red.record_throw(&ten);
            blue.record_throw(&ten);
        }
        Molkky::new(vec![red, blue]);
    }

    #[test]
    fn 存在しないチームへの記録を拒否して得点を保つ() {
        let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
        game.record_throw(0, &Throw::new(&[Skittle::new(5)]));
        for index in [2, usize::MAX] {
            let failed = catch_unwind(AssertUnwindSafe(|| {
                game.record_throw(index, &Throw::new(&[Skittle::new(12)]));
            }));
            assert!(failed.is_err());
            assert_eq!(game.teams()[0].score().history(), &[5]);
            assert!(game.teams()[1].score().history().is_empty());
            assert_eq!(game.result(), None);
        }
    }

    #[test]
    fn 終了後はどのチームへの記録も拒否して履歴と勝者を保つ() {
        let mut game = game_at_forty_nine();
        game.record_throw(1, &Throw::new(&[Skittle::new(1)]));
        for index in [0, 1] {
            let failed = catch_unwind(AssertUnwindSafe(|| {
                game.record_throw(index, &Throw::new(&[Skittle::new(12)]));
            }));
            assert!(failed.is_err());
            assert!(game.teams()[0].score().history().is_empty());
            assert_eq!(game.teams()[1].score().history(), &[12, 12, 12, 12, 1, 1]);
            assert_eq!(game.teams()[1].score().value(), 50);
            assert_eq!(game.result(), Some(1));
        }
    }
}

mod target_score {
    use super::*;

    #[test]
    fn 四十九点ではゲームは終了しない() {
        let game = game_at_forty_nine();
        assert_eq!(game.teams()[1].score().value(), 49);
        assert_eq!(game.result(), None);
    }

    #[test]
    fn 五十点ちょうどで該当チームの勝利になる() {
        let mut game = game_at_forty_nine();
        game.record_throw(1, &Throw::new(&[Skittle::new(1)]));
        assert_eq!(game.teams()[1].score().value(), 50);
        assert_eq!(game.result(), Some(1));
    }

    #[test]
    fn 超過時は25点に戻してゲームを継続する() {
        let mut game = game_at_forty_nine();
        assert_eq!(game.record_throw(1, &Throw::new(&[Skittle::new(2)])), 2);
        assert_eq!(game.teams()[1].score().value(), 25);
        assert_eq!(game.result(), None);
        game.record_throw(1, &Throw::new(&[Skittle::new(12)]));
        assert_eq!(game.teams()[1].score().value(), 37);
        game.record_throw(0, &Throw::new(&[Skittle::new(5)]));
        assert_eq!(game.teams()[0].score().value(), 5);
        game.record_throw(1, &Throw::new(&[Skittle::new(12)]));
        game.record_throw(1, &Throw::new(&[Skittle::new(1)]));
        assert_eq!(game.teams()[1].score().value(), 50);
        assert_eq!(game.result(), Some(1));
    }
}

#[test]
fn 一チームのゲームと既に勝者がいるゲームも表現できる() {
    assert_eq!(Molkky::new(vec![Team::new("赤")]).result(), None);
    let mut winner = Team::new("赤");
    let ten = Throw::new(&[Skittle::new(10)]);
    for _ in 0..5 {
        winner.record_throw(&ten);
    }
    assert_eq!(Molkky::new(vec![winner]).result(), Some(0));
}
