use crate::cui::models::{CuiSkittle, CuiStick};
use crate::{
    Molkky, Skittle, SkittleState, Stick, Throw,
    cui::{models::CuiField, services::CuiMolkky},
};

mod flying {
    use super::*;

    #[test]
    fn 飛行待ちをせず確率を保った棒で命中と採点を行う() {
        let mut rules = CuiMolkky::new(vec![5]);
        let stick = CuiStick::default().throw(1.0);
        let original_id = stick.as_ref().id();
        assert!(!Molkky::fly(&mut rules, &stick));
        assert_eq!(stick.probability(), 1.0);
        assert_eq!(stick.as_ref().id(), original_id);
        let pin = CuiSkittle::from(Skittle::new(5));
        let fallen = rules.hit(&stick, &pin);
        assert_eq!(fallen.as_ref().state(), SkittleState::Fall);
        assert_eq!(rules.calc_score(&[*fallen.as_ref()]).value(), 5);
    }
}

mod aiming {
    use super::*;

    #[test]
    fn 指定番号の既存個体を入力順に選んで返す() {
        let field = CuiField::default();
        let rules = CuiMolkky::new(vec![12, 5]);
        let rules: &dyn Molkky<Stick = CuiStick, Skittle = CuiSkittle> = &rules;
        assert_eq!(
            rules.aim(&CuiStick::new(Stick::new(), 1.0), &field.skittles),
            [field.skittles[11], field.skittles[4]]
        );
    }

    #[test]
    fn 零本の選択は投てきミスとして零点になる() {
        let rules = CuiMolkky::new(Vec::new());
        let stick = CuiStick::new(Stick::new(), 1.0);
        let targets = rules.aim(&stick, &CuiField::default().skittles);
        let fallen = fallen_skittles(&rules, &stick, &targets);
        assert!(fallen.is_empty());
        assert_eq!(rules.calc_score(&fallen).value(), 0);
    }
}

mod throwing {
    use super::*;

    #[test]
    fn 選んだ個体だけを転倒結果として返し元の十二本と個体idを維持する() {
        let field = CuiField::default();
        let original = field.skittles.clone();
        let rules = CuiMolkky::new(vec![5]);
        let stick = CuiStick::new(Stick::new(), 1.0);
        let targets = rules.aim(&stick, &field.skittles);
        let fallen = fallen_skittles(&rules, &stick, &targets);
        assert_eq!(
            fallen,
            [field.skittles[4].as_ref().with_state(SkittleState::Fall)]
        );
        assert_eq!(rules.calc_score(&fallen).value(), 5);
        assert_eq!(field.skittles, original);
        assert_eq!(targets[0].as_ref().state(), SkittleState::Stand);
    }

    #[test]
    fn 次の投てきも同じフィールドから選び前回の転倒を持ち越さない() {
        let field = CuiField::default();
        let stick = CuiStick::new(Stick::new(), 1.0);
        let first = CuiMolkky::new(vec![5]);
        let second = CuiMolkky::new(vec![8, 12]);
        let targets = first.aim(&stick, &field.skittles);
        let fallen = fallen_skittles(&first, &stick, &targets);
        assert_eq!(first.calc_score(&fallen).value(), 5);
        let targets = second.aim(&stick, &field.skittles);
        let fallen = fallen_skittles(&second, &stick, &targets);
        assert_eq!(
            fallen
                .iter()
                .map(|skittle| skittle.id())
                .collect::<Vec<_>>(),
            [
                field.skittles[7].as_ref().id(),
                field.skittles[11].as_ref().id()
            ]
        );
        assert_eq!(second.calc_score(&fallen).value(), 2);
        assert_eq!(field.skittles.len(), 12);
        assert!(
            field
                .skittles
                .iter()
                .all(|skittle| skittle.as_ref().state() == SkittleState::Stand)
        );
    }
}

mod winner_selection {
    use super::*;
    use crate::{Name, ScoreBoard, Team};

    #[test]
    fn 他チームの失格で勝者を選んでも元の投てき履歴は変更しない() {
        let rules = CuiMolkky::default();
        let history = [5, 8]
            .map(|number| rules.calc_score(&[crate::Skittle::new(number)]))
            .to_vec();
        let original = ScoreBoard::new(history.clone());
        let disqualified = ScoreBoard::new(vec![rules.calc_score(&[]); 3]);
        let teams = [
            Team::new(Name::new("赤")).with_score_board(disqualified),
            Team::new(Name::new("青")).with_score_board(original.clone()),
        ];
        let winner = rules.judge_game(&teams).unwrap();
        assert!(std::ptr::eq(winner, &teams[1]));
        assert_eq!(winner.score_board().history(), history);
        assert_eq!(winner.score_board().total(), 13);
        assert_eq!(original.history(), history);
        assert_eq!(original.total(), 13);
    }
}

mod probability {
    use super::*;

    #[test]
    fn 確率零なら狙った個体は一本も倒れない() {
        let field = CuiField::default();
        let original = field.skittles.clone();
        let rules = CuiMolkky::new((1..=12).collect());
        let stick = CuiStick::new(Stick::new(), 0.0);
        let targets = rules.aim(&stick, &field.skittles);
        assert_eq!(targets.len(), 12);
        assert!(fallen_skittles(&rules, &stick, &targets).is_empty());
        assert_eq!(field.skittles, original);
    }
}

mod chaining {
    use super::*;

    #[test]
    fn 棒の確率を保持して連鎖ごとに二十ポイント下げ同番号の別個体へ引き継ぐ() {
        let rules = CuiMolkky {
            random: || 0.3,
            ..CuiMolkky::default()
        };
        let stick = CuiStick::default().throw(0.8);
        let original = CuiSkittle::from(Skittle::new(5));
        let first = rules.hit(&stick, &original);
        assert_eq!(first.probability(), 0.8);
        assert_eq!(first.as_ref().state(), SkittleState::Fall);
        let target = CuiSkittle::from(Skittle::new(5));
        let second = rules.chain_hit(&first, &target);
        assert_eq!(
            *second.as_ref(),
            target.as_ref().with_state(SkittleState::Fall)
        );
        assert!((second.probability() - 0.6).abs() < 1e-12);
        assert_ne!(second.as_ref().id(), first.as_ref().id());
        let third = rules.chain_hit(&second, &CuiSkittle::from(Skittle::new(8)));
        assert_eq!(third.as_ref().state(), SkittleState::Fall);
        assert!((third.probability() - 0.4).abs() < 1e-12);
        let stopped = CuiSkittle::from(Skittle::new(12));
        assert_eq!(rules.chain_hit(&third, &stopped), stopped);
        assert_eq!(original.as_ref().state(), SkittleState::Stand);
        assert_eq!(target.as_ref().state(), SkittleState::Stand);
        assert_eq!(
            rules
                .calc_score(&[*first.as_ref(), *second.as_ref(), *third.as_ref()])
                .value(),
            3
        );
    }

    #[test]
    fn 連鎖失敗時は対象の状態も確率も変えない() {
        let rules = CuiMolkky {
            random: || 0.9,
            ..CuiMolkky::default()
        };
        let fallen = CuiSkittle::new(Skittle::new(5).with_state(SkittleState::Fall), 0.8);
        let target = CuiSkittle::from(Skittle::new(8));
        assert_eq!(rules.chain_hit(&fallen, &target), target);
        assert_eq!(fallen.probability(), 0.8);
    }

    #[test]
    fn 二十パーセント以下からの連鎖は確率を零で止める() {
        let rules = CuiMolkky {
            random: || 0.0,
            ..CuiMolkky::default()
        };
        let target = CuiSkittle::from(Skittle::new(8));
        for probability in [0.0, 0.1, 0.2] {
            let fallen =
                CuiSkittle::new(Skittle::new(5).with_state(SkittleState::Fall), probability);
            assert_eq!(rules.chain_hit(&fallen, &target), target);
        }
    }

    #[test]
    fn 連鎖を重ねても浮動小数の残差で余分な一本を倒さない() {
        let rules = CuiMolkky {
            random: || 0.0,
            ..CuiMolkky::default()
        };
        let mut fallen = CuiSkittle::new(Skittle::new(5).with_state(SkittleState::Fall), 0.8);
        for expected in [0.6, 0.4, 0.2] {
            let target = CuiSkittle::from(Skittle::new(8));
            fallen = rules.chain_hit(&fallen, &target);
            assert_eq!(fallen.as_ref().state(), SkittleState::Fall);
            assert!((fallen.probability() - expected).abs() < 1e-12);
        }
        let target = CuiSkittle::from(Skittle::new(12));
        assert_eq!(rules.chain_hit(&fallen, &target), target);
    }

    #[test]
    fn 立っている連鎖元や既に倒れた対象は再判定しない() {
        let rules = CuiMolkky {
            random: || panic!("乱数を再評価しない"),
            ..CuiMolkky::default()
        };
        let standing = CuiSkittle::from(Skittle::new(5));
        let fallen = CuiSkittle::new(Skittle::new(8).with_state(SkittleState::Fall), 0.8);
        assert_eq!(rules.chain_hit(&standing, &standing), standing);
        assert_eq!(rules.chain_hit(&fallen, &fallen), fallen);
        assert_eq!(rules.hit(&CuiStick::default(), &fallen), fallen);
    }
}

fn fallen_skittles<M: Molkky + ?Sized>(
    rules: &M,
    stick: &M::Stick,
    skittles: &[M::Skittle],
) -> Vec<Skittle> {
    skittles
        .iter()
        .map(|skittle| rules.hit(stick, skittle))
        .filter(|skittle| skittle.as_ref().state() == SkittleState::Fall)
        .map(|skittle| *skittle.as_ref())
        .collect()
}
