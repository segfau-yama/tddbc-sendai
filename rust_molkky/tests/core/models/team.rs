use rust_molkky::{Name, Point, ScoreBoard, Team};

mod creation {
    use super::*;

    #[test]
    fn 名前を保持し得点表は零点から始まる() {
        let team = Team::new(Name::new("赤チーム").unwrap());
        assert_eq!(team.name().as_str(), "赤チーム");
        assert_eq!(team.score_board().total(), 0);
        assert!(team.score_board().history().is_empty());
    }
}

mod independent_scores {
    use super::*;

    #[test]
    fn 得点表を差し替えても元のチームは変わらない() {
        let original = Team::new(Name::new("赤").unwrap());
        let updated = original.with_score_board(ScoreBoard::new(vec![Point::new(11)]));
        assert_eq!(original.score_board().total(), 0);
        assert!(original.score_board().history().is_empty());
        assert_eq!(updated.score_board().total(), 11);
        assert_eq!(updated.name(), original.name());
        assert_eq!(updated.id(), original.id());
    }

    #[test]
    fn チームごとに得点表を独立して保持する() {
        let red = Team::new(Name::new("赤").unwrap())
            .with_score_board(ScoreBoard::new(vec![Point::new(5)]));
        let blue = Team::new(Name::new("青").unwrap());
        let updated = red.with_score_board(red.score_board().add(Point::new(11)));
        assert_eq!(red.score_board().total(), 5);
        assert_eq!(updated.score_board().total(), 16);
        assert_eq!(blue.score_board().total(), 0);
    }
}

mod identity {
    use super::*;

    #[test]
    fn 同じ名前でも別のチームとして識別できる() {
        let first = Team::new(Name::new("同じ名前").unwrap());
        let second = Team::new(Name::new("同じ名前").unwrap());
        assert_eq!(first.name(), second.name());
        assert_ne!(first.id(), second.id());
        assert_eq!(first.clone().id(), first.id());
    }

    #[test]
    fn 一覧の順番を変えても識別子で元のチームを参照できる() {
        let red = Team::new(Name::new("赤").unwrap());
        let id = red.id();
        let mut teams = [red, Team::new(Name::new("青").unwrap())];
        teams.swap(0, 1);
        assert_eq!(
            teams
                .iter()
                .find(|team| team.id() == id)
                .unwrap()
                .name()
                .as_str(),
            "赤"
        );
    }
}

mod turn {
    use super::*;

    #[test]
    fn 手番だけを更新し元の値と得点を保持する() {
        let original = Team::new(Name::new("赤").unwrap());
        let active = original.with_turn(true);
        assert!(!original.turn());
        assert!(active.turn());
        assert_eq!(active.id(), original.id());
        let scored = active.with_score_board(ScoreBoard::new(vec![Point::new(5)]));
        assert!(scored.turn());
        assert_eq!(scored.clone().id(), active.id());
        assert_eq!(scored.score_board().total(), 5);
    }
}
