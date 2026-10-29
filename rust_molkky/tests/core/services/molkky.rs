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
    Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
}

#[test]
fn 空の実装を使い使用側で得点表を更新できる() {
    struct Rules;
    impl Molkky for Rules {}
    let original = team("赤", &[]).with_turn(true);
    let updated = Rules.hit(
        &rust_molkky::Stick::new(),
        &[Skittle::new(12)],
        &mut |_, pins| pins.to_vec(),
    );
    let scored = original.with_score_board(original.score_board().add(Rules.calc_score(&updated)));
    assert_eq!(original.score_board().total(), 0);
    assert_eq!(scored.score_board().total(), 12);
    assert_eq!(scored.id(), original.id());
    assert!(scored.turn());
}

#[test]
fn 判定関数に棒と全個体を一度渡し更新後も全個体を返す() {
    use rust_molkky::{JudgeStrategy, SkittleState, Stick};
    let stick = Stick::new();
    let pins = [Skittle::new(12), Skittle::new(12)];
    let mut calls = 0;
    let mut judge = |received: &Stick, skittles: &[Skittle]| {
        calls += 1;
        assert_eq!(received.id(), stick.id());
        assert_eq!(skittles, pins);
        vec![skittles[0].with_state(SkittleState::Fall), skittles[1]]
    };
    let strategy: JudgeStrategy<'_> = &mut judge;
    let rules: &dyn Molkky = &SimpleMolkky;
    let updated = rules.hit(&stick, &pins, strategy);
    assert_eq!(calls, 1);
    assert_eq!(updated.len(), 2);
    assert_eq!(updated[0].id(), pins[0].id());
    assert_eq!(updated[0].state(), SkittleState::Fall);
    assert_eq!(updated[1], pins[1]);
    assert!(pins.iter().all(|pin| pin.state() == SkittleState::Stand));
}

#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 重複した入力は判定関数を呼ぶ前に拒否する() {
    let pin = Skittle::new(5);
    SimpleMolkky.hit(&rust_molkky::Stick::new(), &[pin, pin], &mut |_, _| {
        panic!("呼び出されない")
    });
}

#[test]
fn 判定後の重複と個体の追加と欠落と置換を拒否する() {
    use rust_molkky::Stick;
    let pins = [Skittle::new(5), Skittle::new(5)];
    for invalid in [
        vec![pins[0], pins[0]],
        vec![pins[0]],
        vec![pins[0], pins[1], Skittle::new(5)],
        vec![pins[0], Skittle::new(5)],
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, _| invalid.clone());
            })
            .is_err()
        );
    }
}

#[test]
fn 判定は空の集合と並び替えられた同じ個体集合を返せる() {
    use rust_molkky::Stick;
    assert!(
        SimpleMolkky
            .hit(&Stick::new(), &[], &mut |_, pins| pins.to_vec())
            .is_empty()
    );
    let pins = [Skittle::new(5), Skittle::new(8)];
    let updated = SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, pins| {
        pins.iter().rev().copied().collect()
    });
    assert_eq!(updated, [pins[1], pins[0]]);
}

#[test]
fn 判定結果は入力と判定関数の借用期間後も保持できる() {
    use rust_molkky::{SkittleState, Stick};
    let (updated, id) = {
        let stick = Stick::new();
        let pins = [Skittle::new(12)];
        let mut calls = 0;
        let mut judge = |_: &Stick, pins: &[Skittle]| {
            calls += 1;
            vec![pins[0].with_state(SkittleState::Fall)]
        };
        let updated = SimpleMolkky.hit(&stick, &pins, &mut judge);
        assert_eq!(calls, 1);
        assert_eq!(pins[0].state(), SkittleState::Stand);
        (updated, pins[0].id())
    };
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].id(), id);
    assert_eq!(updated[0].state(), SkittleState::Fall);
}

mod validation_and_scoring {
    use super::*;

    #[test]
    fn 検証の後も元の配列を更新できる() {
        let mut pins = [Skittle::new(5)];
        SimpleMolkky.validate_skittles(&pins);
        pins[0] = Skittle::new(12);
        assert_eq!(pins[0].number(), 12);
    }

    #[test]
    fn 番号でなく個体の重複を検証し入力を変えない() {
        let pins = [Skittle::new(12), Skittle::new(12)];
        let original = pins;
        SimpleMolkky.validate_skittles(&[]);
        SimpleMolkky.validate_skittles(&pins);
        assert_eq!(pins, original);
        assert!(
            std::panic::catch_unwind(|| {
                SimpleMolkky.validate_skittles(&[pins[0], pins[0]]);
            })
            .is_err()
        );
    }

    #[test]
    #[should_panic(expected = "スキットルIDが重複しています")]
    fn 同じ個体は状態が異なっても二重登録できない() {
        let pin = Skittle::new(5);
        SimpleMolkky.validate_skittles(&[pin, pin.with_state(rust_molkky::SkittleState::Fall)]);
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
}

mod team_selection {
    use super::*;

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
        Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
    }

    #[test]
    fn 次の手番へ循環し失格を飛ばして元のチームを変更しない() {
        let teams = [
            team("赤", &[]).with_turn(true),
            team("青", &[0; 3]),
            team("緑", &[5]),
        ];
        let next = SimpleMolkky.next_team(&teams);
        assert_eq!(
            next.iter().map(Team::turn).collect::<Vec<_>>(),
            [false, false, true]
        );
        assert_eq!(
            teams.iter().map(Team::turn).collect::<Vec<_>>(),
            [true, false, false]
        );
        for (before, after) in teams.iter().zip(&next) {
            assert_eq!(before.id(), after.id());
            assert_eq!(before.name(), after.name());
            assert_eq!(
                before.score_board().history(),
                after.score_board().history()
            );
            assert_eq!(before.score_board().state(), after.score_board().state());
        }
        let wrapped = SimpleMolkky.next_team(&next);
        assert_eq!(
            wrapped.iter().map(Team::turn).collect::<Vec<_>>(),
            [true, false, false]
        );
    }

    #[test]
    fn 手番未設定なら先頭の有効チームで開始し単独参加なら同じチームへ戻る() {
        let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[])];
        let next = SimpleMolkky.next_team(&teams);
        assert_eq!(
            next.iter().map(Team::turn).collect::<Vec<_>>(),
            [false, true, false]
        );
        let solo = SimpleMolkky.next_team(&[team("赤", &[])]);
        assert!(solo[0].turn());
        assert!(SimpleMolkky.next_team(&solo)[0].turn());
    }

    #[test]
    fn 空と全失格と勝利と最後の生存者では手番を解除する() {
        assert!(SimpleMolkky.next_team(&[]).is_empty());
        for teams in [
            vec![team("赤", &[0; 3]).with_turn(true)],
            vec![team("赤", &[0; 3]), team("青", &[0; 3]).with_turn(true)],
            vec![team("赤", &[10; 5]).with_turn(true), team("青", &[])],
            vec![team("赤", &[0; 3]).with_turn(true), team("青", &[])],
        ] {
            let next = SimpleMolkky.next_team(&teams);
            assert!(next.iter().all(|team| !team.turn()));
            assert_eq!(next.len(), teams.len());
        }
    }

    #[test]
    fn 失格になった現在のチームから次の有効チームへ進める() {
        let teams = [
            team("赤", &[0; 3]).with_turn(true),
            team("青", &[]),
            team("緑", &[]),
        ];
        let next = SimpleMolkky.next_team(&teams);
        assert_eq!(
            next.iter().map(Team::turn).collect::<Vec<_>>(),
            [false, true, false]
        );
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
    #[should_panic(expected = "チームIDが重複しています")]
    fn 手番の更新でも同じチームの二重登録を拒否する() {
        let first = team("赤", &[]);
        SimpleMolkky.next_team(&[first.clone(), first]);
    }

    #[test]
    #[should_panic(expected = "チームは1件以上必要です")]
    fn 空の参加者で勝敗を判定できない() {
        SimpleMolkky.judge_game(&[]);
    }
}
