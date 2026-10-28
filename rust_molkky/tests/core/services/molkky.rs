use rust_molkky::{Molkky, Name, ScoreBoard, ScoreState, SimpleMolkky, Skittle, Team};

fn team(name: &str, points: &[i32]) -> Team {
    let history = points
        .iter()
        .map(|&n| {
            if n == 0 {
                SimpleMolkky.calc_score(&[])
            } else {
                SimpleMolkky.calc_score(&[Skittle::new(n)])
            }
        })
        .collect();
    Team::new(Name::new(name).unwrap()).with_score_board(ScoreBoard::new(history))
}

#[test]
fn 手番が立ったチームそのものを返し並び順や得点では選ばない() {
    let teams = [
        team("赤", &[]),
        team("青", &[0, 0, 0]).with_turn(true),
        team("緑", &[10; 5]),
    ];
    let rules: &dyn Molkky = &SimpleMolkky;
    let selected = rules.next_team(&teams).unwrap();
    assert!(std::ptr::eq(selected, &teams[1]));
    assert_eq!(selected.score_board().state(), ScoreState::Disqualified);
    assert!(selected.turn());
}

#[test]
fn 手番がなければ空や勝者の有無にかかわらず返さない() {
    assert!(SimpleMolkky.next_team(&[]).is_none());
    assert!(SimpleMolkky.next_team(&[team("赤", &[])]).is_none());
    assert!(SimpleMolkky.next_team(&[team("赤", &[10; 5])]).is_none());
}

#[test]
#[should_panic(expected = "手番のチームが複数います")]
fn 複数の手番を拒否する() {
    SimpleMolkky.next_team(&[
        team("赤", &[]).with_turn(true),
        team("青", &[]).with_turn(true),
    ]);
}

#[test]
fn 勝利状態のチームそのものを返し手番には依存しない() {
    let teams = [team("赤", &[]).with_turn(true), team("青", &[10; 5])];
    let winner = SimpleMolkky.judge_game(&teams).unwrap();
    assert!(std::ptr::eq(winner, &teams[1]));
    assert_eq!(winner.score_board().state(), ScoreState::Won);
    assert!(!winner.turn());
}

#[test]
fn 勝利状態でなければ最後の生存者や全失格でも勝者を返さない() {
    for teams in [
        vec![team("赤", &[])],
        vec![team("赤", &[0; 3]), team("青", &[])],
        vec![team("赤", &[0; 3])],
    ] {
        assert!(SimpleMolkky.judge_game(&teams).is_none());
    }
}

#[test]
#[should_panic(expected = "勝者が複数います")]
fn 複数の勝利状態を拒否する() {
    SimpleMolkky.judge_game(&[team("赤", &[10; 5]), team("青", &[10; 5])]);
}

#[test]
#[should_panic(expected = "チームIDが重複しています")]
fn 同じチームの二重登録を拒否する() {
    let first = team("赤", &[]);
    SimpleMolkky.judge_game(&[first.clone(), first]);
}

#[test]
#[should_panic(expected = "チームは1件以上必要です")]
fn 空の参加者で勝敗を判定できない() {
    SimpleMolkky.judge_game(&[]);
}

#[test]
fn 空の実装を使い使用側で得点表を更新できる() {
    struct Rules;
    impl Molkky for Rules {}
    let original = team("赤", &[]).with_turn(true);
    let scored = original.with_score_board(
        original
            .score_board()
            .add(Rules.calc_score(&[Skittle::new(12)])),
    );
    assert_eq!(original.score_board().total(), 0);
    assert_eq!(scored.score_board().total(), 12);
    assert_eq!(scored.id(), original.id());
    assert!(scored.turn());
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
