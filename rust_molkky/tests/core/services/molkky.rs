use rust_molkky::{
    JudgeStrategy, Molkky, Name, ScoreBoard, ScoreState, SimpleMolkky, Skittle, SkittleState,
    Stick, Team,
};

fn team(name: &str, points: &[i32]) -> Team {
    let history = points
        .iter()
        .map(|&number| match number {
            0 => SimpleMolkky.calc_score(&[]),
            number => SimpleMolkky.calc_score(&[Skittle::new(number)]),
        })
        .collect();
    Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
}

mod scoring {
    use super::*;

    mod one_skittle {
        use super::*;

        #[test]
        fn s01_1_番号一の一本なら一点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(1)]).value(), 1);
        }

        #[test]
        fn s01_1_番号五の一本なら五点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(5)]).value(), 5);
        }

        #[test]
        fn s01_1_番号十二の一本なら十二点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[Skittle::new(12)]).value(), 12);
        }
    }

    mod multiple_skittles {
        use super::*;

        #[test]
        fn s01_2_番号五と八の二本なら番号の合計ではなく二点になる() {
            let pins = [Skittle::new(5), Skittle::new(8)];
            assert_eq!(SimpleMolkky.calc_score(&pins).value(), 2);
        }

        #[test]
        fn s01_2_異なる番号の三本なら三点になる() {
            let pins = [Skittle::new(1), Skittle::new(5), Skittle::new(12)];
            assert_eq!(SimpleMolkky.calc_score(&pins).value(), 3);
        }

        #[test]
        fn s01_3_一から十二の標準の十二本なら十二点になる() {
            let pins: Vec<_> = (1..=12).map(Skittle::new).collect();
            assert_eq!(SimpleMolkky.calc_score(&pins).value(), 12);
        }
    }

    mod no_skittles {
        use super::*;

        #[test]
        fn s01_3_一本も倒さなければ零点になる() {
            assert_eq!(SimpleMolkky.calc_score(&[]).value(), 0);
        }
    }

    mod ownership {
        use super::*;

        #[test]
        fn 元の配列を更新しても取得した点数は変わらない() {
            let mut pins = [Skittle::new(5)];
            let point = SimpleMolkky.calc_score(&pins);
            pins[0] = Skittle::new(12);
            assert_eq!(pins[0].number(), 12);
            assert_eq!(point.value(), 5);
        }
    }
}

mod hit {
    use super::*;

    mod delegation {
        use super::*;

        #[test]
        fn 判定に棒と全個体を一度渡し結果をそのまま返す() {
            let stick = Stick::new();
            let pins = [Skittle::new(5), Skittle::new(8)];
            let mut calls = 0;
            let mut judge = |received: &Stick, received_pins: &[Skittle]| {
                calls += 1;
                assert_eq!(received.id(), stick.id());
                assert_eq!(received_pins, pins);
                vec![
                    received_pins[0].with_state(SkittleState::Fall),
                    received_pins[1],
                ]
            };
            let strategy: JudgeStrategy<'_> = &mut judge;
            let rules: &dyn Molkky = &SimpleMolkky;
            let updated = rules.hit(&stick, &pins, strategy);
            assert_eq!(calls, 1);
            assert_eq!(updated, [pins[0].with_state(SkittleState::Fall), pins[1]]);
            assert!(pins.iter().all(|pin| pin.state() == SkittleState::Stand));
        }

        #[test]
        fn 空集合でも判定を一度呼ぶ() {
            let mut calls = 0;
            let updated = SimpleMolkky.hit(&Stick::new(), &[], &mut |_, pins| {
                calls += 1;
                assert!(pins.is_empty());
                Vec::new()
            });
            assert_eq!(calls, 1);
            assert!(updated.is_empty());
        }

        #[test]
        fn 判定が返した個体の並び順を保つ() {
            let pins = [Skittle::new(5), Skittle::new(8)];
            let updated = SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, pins| {
                pins.iter().rev().copied().collect()
            });
            assert_eq!(updated, [pins[1], pins[0]]);
        }
    }

    mod ownership {
        use super::*;

        #[test]
        fn 結果は入力と判定関数の借用期間後も保持できる() {
            let (updated, id) = {
                let pins = [Skittle::new(12)];
                let mut judge =
                    |_: &Stick, pins: &[Skittle]| vec![pins[0].with_state(SkittleState::Fall)];
                let updated = SimpleMolkky.hit(&Stick::new(), &pins, &mut judge);
                (updated, pins[0].id())
            };
            assert_eq!(updated.len(), 1);
            assert_eq!(updated[0].id(), id);
            assert_eq!(updated[0].state(), SkittleState::Fall);
        }
    }
}

mod winner_selection {
    use super::*;

    mod last_survivor {
        use super::*;

        #[test]
        fn 二チームで相手が失格したら生存者を返す() {
            let teams = [team("赤", &[0; 3]), team("青", &[5])];
            let winner = SimpleMolkky.judge_game(&teams).unwrap();
            assert!(std::ptr::eq(winner, &teams[1]));
        }

        #[test]
        fn 三チームで他が全て失格したら生存者を返す() {
            let teams = [team("赤", &[0; 3]), team("青", &[5]), team("緑", &[0; 3])];
            let winner = SimpleMolkky.judge_game(&teams).unwrap();
            assert!(std::ptr::eq(winner, &teams[1]));
        }

        #[test]
        fn 単独参加と全失格と生存者が複数の場合は勝者を返さない() {
            for teams in [
                vec![team("赤", &[])],
                vec![team("赤", &[0; 3]), team("青", &[0; 3])],
                vec![team("赤", &[0; 3]), team("青", &[]), team("緑", &[])],
            ] {
                assert!(SimpleMolkky.judge_game(&teams).is_none());
            }
        }
    }

    mod no_winner {
        use super::*;

        #[test]
        fn 空の一覧では該当する勝者がいない() {
            assert!(SimpleMolkky.judge_game(&[]).is_none());
        }

        #[test]
        fn 五十点到達前は勝者を返さない() {
            let teams = [team("赤", &[5]), team("青", &[11])];
            assert!(SimpleMolkky.judge_game(&teams).is_none());
        }
    }

    mod won {
        use super::*;

        #[test]
        fn 手番に依存せず五十点到達済みのチームそのものを返す() {
            let teams = [team("赤", &[5]).with_turn(true), team("青", &[10; 5])];
            let winner = SimpleMolkky.judge_game(&teams).unwrap();
            assert!(std::ptr::eq(winner, &teams[1]));
            assert_eq!(winner.score_board().state(), ScoreState::Won);
            assert_eq!(winner.score_board().total(), 50);
            assert!(!winner.turn());
            assert!(teams[0].turn());
        }

        #[test]
        fn 勝者の参照はサービスの借用期間後も保持できる() {
            let teams = [team("赤", &[10; 5]), team("青", &[])];
            let winner = {
                let rules = SimpleMolkky;
                rules.judge_game(&teams).unwrap()
            };
            assert!(std::ptr::eq(winner, &teams[0]));
        }
    }
}

mod next_team {
    use super::*;

    mod rotation {
        use super::*;

        #[test]
        fn 現在の次のチームを返して元の手番を変更しない() {
            let teams = [
                team("赤", &[]).with_turn(true),
                team("青", &[]),
                team("緑", &[]),
            ];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[1]));
            assert_eq!(
                teams.iter().map(Team::turn).collect::<Vec<_>>(),
                [true, false, false]
            );
        }

        #[test]
        fn 最後のチームの次は先頭へ循環する() {
            let teams = [
                team("赤", &[]),
                team("青", &[]),
                team("緑", &[]).with_turn(true),
            ];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[0]));
        }

        #[test]
        fn 次の失格チームを飛ばして選ぶ() {
            let teams = [
                team("赤", &[]).with_turn(true),
                team("青", &[0; 3]),
                team("緑", &[]),
            ];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[2]));
        }

        #[test]
        fn 現在のチームが失格でも次の有効チームへ進める() {
            let teams = [
                team("赤", &[0; 3]).with_turn(true),
                team("青", &[]),
                team("緑", &[]),
            ];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[1]));
        }
    }

    mod initial_turn {
        use super::*;

        #[test]
        fn 手番未設定なら先頭の有効チームを選ぶ() {
            let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[])];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[1]));
        }

        #[test]
        fn 単独参加なら同じ有効チームを選ぶ() {
            let teams = [team("赤", &[]).with_turn(true)];
            let next = SimpleMolkky.next_team(&teams).unwrap();
            assert!(std::ptr::eq(next, &teams[0]));
        }
    }

    mod no_next_team {
        use super::*;

        #[test]
        fn 空の一覧には次のチームがいない() {
            assert!(SimpleMolkky.next_team(&[]).is_none());
        }

        #[test]
        fn 全失格なら次のチームがいない() {
            let teams = [team("赤", &[0; 3]).with_turn(true), team("青", &[0; 3])];
            assert!(SimpleMolkky.next_team(&teams).is_none());
        }

        #[test]
        fn 五十点到達済みのチームがあれば次の手番を選ばない() {
            let teams = [team("赤", &[10; 5]), team("青", &[]).with_turn(true)];
            assert!(SimpleMolkky.next_team(&teams).is_none());
        }

        #[test]
        fn 複数参加で最後の一チームだけになれば次の手番を選ばない() {
            let teams = [team("赤", &[0; 3]).with_turn(true), team("青", &[])];
            assert!(SimpleMolkky.next_team(&teams).is_none());
        }
    }

    mod ownership {
        use super::*;

        #[test]
        fn 次のチームの参照はサービスの借用期間後も保持できる() {
            let teams = [team("赤", &[]).with_turn(true), team("青", &[])];
            let next = {
                let rules = SimpleMolkky;
                rules.next_team(&teams).unwrap()
            };
            assert!(std::ptr::eq(next, &teams[1]));
        }
    }
}

mod integration_contract {
    use super::*;

    #[test]
    fn 空の実装で投てきし倒れた個体の採点と得点表の更新を使用側で行える() {
        struct Rules;
        impl Molkky for Rules {}
        let original = team("赤", &[]).with_turn(true);
        let pins = [Skittle::new(5), Skittle::new(8)];
        let updated = Rules.hit(&Stick::new(), &pins, &mut |_, pins| {
            vec![pins[0].with_state(SkittleState::Fall), pins[1]]
        });
        let fallen: Vec<_> = updated
            .into_iter()
            .filter(|pin| pin.state() == SkittleState::Fall)
            .collect();
        let scored =
            original.with_score_board(original.score_board().add(Rules.calc_score(&fallen)));
        assert_eq!(original.score_board().total(), 0);
        assert_eq!(scored.score_board().total(), 5);
        assert_eq!(scored.id(), original.id());
        assert!(scored.turn());
    }
}
