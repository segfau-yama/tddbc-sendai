use rust_molkky::gui_2d::services::teams::from_names;

mod initialization {
    use super::*;

    #[test]
    fn 入力順に同名でも別個体のチームを生成する() {
        let teams = from_names(&["赤", "青", "赤"]);
        assert_eq!(
            teams.iter().map(|t| t.name().as_str()).collect::<Vec<_>>(),
            ["赤", "青", "赤"]
        );
        assert_ne!(teams[0].id(), teams[2].id());
        assert!(
            teams
                .iter()
                .all(|t| t.score_board().total() == 0 && !t.turn())
        );
    }
}
