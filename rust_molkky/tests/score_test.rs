use rust_molkky::{Score, Skittle, Throw};

#[test]
fn 一回も投てきしていなければ0点になる() {
    assert_eq!(Score::new(&[]).value(), 0);
    assert!(Score::new(&[]).history().is_empty());
}

#[test]
fn 一回目に5本倒すと5点になる() {
    let first = Throw::new(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]);

    assert_eq!(Score::new(&[&first]).value(), 5);
}

#[test]
fn 五本倒した後に11番を1本倒すと16点になる() {
    let first = Throw::new(&[
        Skittle::new(1),
        Skittle::new(2),
        Skittle::new(3),
        Skittle::new(4),
        Skittle::new(5),
    ]);
    let second = Throw::new(&[Skittle::new(11)]);

    assert_eq!(Score::new(&[&first, &second]).value(), 16);
}

#[test]
fn 零点の投てきは現在の得点を変えない() {
    let first = Throw::new(&[Skittle::new(11)]);
    let missed = Throw::new(&[]);

    assert_eq!(Score::new(&[&first, &missed]).value(), 11);
    assert_eq!(Score::new(&[&first, &missed]).history(), &[11, 0]);
}

#[test]
fn 同じ投てきを2回渡しても2回分合計する() {
    let thrown = Throw::new(&[Skittle::new(11)]);

    assert_eq!(Score::new(&[&thrown, &thrown]).value(), 22);
    assert_eq!(Score::new(&[&thrown, &thrown]).history(), &[11, 11]);
}

#[test]
fn 投てきの点数を順番どおり履歴に保持する() {
    let five = Throw::new(&[Skittle::new(5)]);
    let eleven = Throw::new(&[Skittle::new(11)]);
    assert_eq!(Score::new(&[&five, &eleven]).history(), &[5, 11]);
}

#[test]
fn 投てきを追加しても元の得点と履歴は変わらない() {
    let five = Throw::new(&[Skittle::new(5)]);
    let eleven = Throw::new(&[Skittle::new(11)]);
    let original = Score::new(&[&five]);
    let updated = original.with_throw(&eleven);
    assert_eq!(original.value(), 5);
    assert_eq!(original.history(), &[5]);
    assert_eq!(updated.value(), 16);
    assert_eq!(updated.history(), &[5, 11]);
}

mod target_score {
    use super::{Score, Skittle, Throw};

    fn score_forty_nine() -> Score {
        let twelve = Throw::new(&[Skittle::new(12)]);
        let one = Throw::new(&[Skittle::new(1)]);
        Score::new(&[&twelve, &twelve, &twelve, &twelve, &one])
    }

    #[test]
    fn 四十九点はそのまま保持する() {
        assert_eq!(score_forty_nine().value(), 49);
    }

    #[test]
    fn 五十点ちょうどはそのまま保持する() {
        let one = Throw::new(&[Skittle::new(1)]);
        assert_eq!(score_forty_nine().with_throw(&one).value(), 50);
    }

    #[test]
    fn 五十点を超えた投てきで25点に戻る() {
        let two = Throw::new(&[Skittle::new(2)]);
        let score = score_forty_nine().with_throw(&two);
        assert_eq!(score.value(), 25);
        assert_eq!(score.history(), &[12, 12, 12, 12, 1, 2]);
    }

    #[test]
    fn 超過後の点数は25点から加算し再び超過しても25点に戻る() {
        let two = Throw::new(&[Skittle::new(2)]);
        let twelve = Throw::new(&[Skittle::new(12)]);
        let score = score_forty_nine().with_throw(&two);
        let thirty_seven = score.with_throw(&twelve);
        let forty_nine = thirty_seven.with_throw(&twelve);
        let reset_again = forty_nine.with_throw(&two);
        assert_eq!(thirty_seven.value(), 37);
        assert_eq!(forty_nine.value(), 49);
        assert_eq!(reset_again.value(), 25);
        let replayed = Score::new(&[
            &twelve,
            &twelve,
            &twelve,
            &twelve,
            &Throw::new(&[Skittle::new(1)]),
            &two,
            &twelve,
            &twelve,
            &two,
        ]);
        assert_eq!(replayed.value(), reset_again.value());
        assert_eq!(replayed.history(), reset_again.history());
    }
}
