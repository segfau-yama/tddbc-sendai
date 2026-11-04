use rust_molkky::cui::services::teams::{add_name, from_names};

mod names {
    use super::*;

    #[test]
    fn 前後の空白を除き日本語と内部の空白と記号を保つ() {
        let mut teams = Vec::new();
        add_name(&mut teams, "　 赤 チーム! 🏆  ").unwrap();
        assert_eq!(teams[0].name().as_str(), "赤 チーム! 🏆");
        assert_eq!(teams[0].score_board().total(), 0);
    }

    #[test]
    fn 空の名前を拒否してチームを追加しない() {
        let mut teams = Vec::new();
        for name in ["", " ", "　"] {
            assert!(add_name(&mut teams, name).is_err());
            assert!(teams.is_empty());
        }
        add_name(&mut teams, "赤").unwrap();
        assert_eq!(teams.len(), 1);
    }

    #[test]
    fn 日本語三十文字は許可し三十一文字を拒否する() {
        let mut teams = Vec::new();
        assert!(add_name(&mut teams, &"赤".repeat(31)).is_err());
        add_name(&mut teams, &"赤".repeat(30)).unwrap();
        assert_eq!(teams[0].name().as_str(), "赤".repeat(30));
    }

    #[test]
    fn trimで消える前後の制御文字も拒否する() {
        let mut teams = Vec::new();
        for name in [
            "赤\n青",
            "\t赤",
            "赤\r",
            "赤\0",
            "赤\x1b[31m",
            "赤\u{7f}",
            "赤\u{85}",
        ] {
            assert!(add_name(&mut teams, name).is_err());
            assert!(teams.is_empty());
        }
    }

    #[test]
    fn 同名でも別のチームを入力順に作る() {
        let names = vec![" 赤 ".to_owned(), "赤".to_owned()];
        let teams = from_names(&names).unwrap();
        assert_eq!(teams[0].name(), teams[1].name());
        assert_ne!(teams[0].id(), teams[1].id());
    }
}
