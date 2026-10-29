use rust_molkky::{
    Molkky, SkittleState, Stick,
    cui::{models::Field, services::CuiMolkky},
};

mod aiming {
    use super::*;

    #[test]
    fn 指定番号の既存個体を入力順に選んで返す() {
        let field = Field::default();
        let rules = CuiMolkky::new(vec![12, 5]);
        let rules: &dyn Molkky = &rules;
        assert_eq!(
            rules.aim(&Stick::new(), &field.skittles),
            [field.skittles[11], field.skittles[4]]
        );
    }

    #[test]
    fn 零本の選択は投てきミスとして零点になる() {
        let rules = CuiMolkky::new(Vec::new());
        let stick = Stick::new();
        let targets = rules.aim(&stick, &Field::default().skittles);
        let fallen = rules.hit(&stick, &targets);
        assert!(fallen.is_empty());
        assert_eq!(rules.calc_score(&fallen).value(), 0);
    }
}

mod throwing {
    use super::*;

    #[test]
    fn 選んだ個体だけを転倒結果として返し元の十二本と個体idを維持する() {
        let field = Field::default();
        let original = field.skittles.clone();
        let rules = CuiMolkky::new(vec![5]);
        let stick = Stick::new();
        let targets = rules.aim(&stick, &field.skittles);
        let fallen = rules.hit(&stick, &targets);
        assert_eq!(fallen, [field.skittles[4].with_state(SkittleState::Fall)]);
        assert_eq!(rules.calc_score(&fallen).value(), 5);
        assert_eq!(field.skittles, original);
        assert_eq!(targets[0].state(), SkittleState::Stand);
    }

    #[test]
    fn 次の投てきも同じフィールドから選び前回の転倒を持ち越さない() {
        let field = Field::default();
        let stick = Stick::new();
        let first = CuiMolkky::new(vec![5]);
        let second = CuiMolkky::new(vec![8, 12]);
        let targets = first.aim(&stick, &field.skittles);
        let fallen = first.hit(&stick, &targets);
        assert_eq!(first.calc_score(&fallen).value(), 5);
        let targets = second.aim(&stick, &field.skittles);
        let fallen = second.hit(&stick, &targets);
        assert_eq!(
            fallen.iter().map(|pin| pin.id()).collect::<Vec<_>>(),
            [field.skittles[7].id(), field.skittles[11].id()]
        );
        assert_eq!(second.calc_score(&fallen).value(), 2);
        assert_eq!(field.skittles.len(), 12);
        assert!(
            field
                .skittles
                .iter()
                .all(|pin| pin.state() == SkittleState::Stand)
        );
    }
}

mod winner_selection {
    use super::*;
    use rust_molkky::{Name, ScoreBoard, Team};

    #[test]
    fn 他チームの失格で勝者を選んでも元の投てき履歴は変更しない() {
        let rules = CuiMolkky::default();
        let history = [5, 8]
            .map(|number| rules.calc_score(&[rust_molkky::Skittle::new(number)]))
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
