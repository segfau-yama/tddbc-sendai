use rust_molkky::{
    Name, Team,
    gui_2d::models::{Game, GamePhase},
};

mod initialization {
    use super::*;

    #[test]
    fn チームを保持して共通フィールドと棒を用意し待機から始める() {
        let teams = vec![
            Team::new(Name::new("赤")).with_turn(true),
            Team::new(Name::new("青")),
        ];
        let ids: Vec<_> = teams.iter().map(Team::id).collect();
        let game = Game::new(teams);

        assert_eq!(game.teams.iter().map(Team::id).collect::<Vec<_>>(), ids);
        assert!(game.teams[0].turn());
        assert!(!game.teams[1].turn());
        assert_eq!(game.field.skittles().len(), 12);
        assert_eq!(game.stick.target(), None);
        assert_eq!(game.stick.speed(), 0.0);
        assert_eq!(game.phase, GamePhase::Ready);
    }
}
