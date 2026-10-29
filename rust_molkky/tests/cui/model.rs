use rust_molkky::cui::model::{TeamSetup, ThrowInput};

mod teams {
    use super::*;

    mod count {
        use super::*;

        #[test]
        fn チーム数を空白付きの文字列から変換する() {
            let mut setup = TeamSetup::parse_count(" 2 ").unwrap();
            setup.add_name("赤").unwrap();
            assert!(!setup.is_complete());
            setup.add_name("青").unwrap();
            assert!(setup.is_complete());
            assert_eq!(setup.finish().unwrap().len(), 2);
        }

        #[test]
        fn 一チーム以上で構築でき零チームを拒否する() {
            assert!(TeamSetup::new(1).is_ok());
            assert!(TeamSetup::new(3).is_ok());
            assert!(TeamSetup::new(0).is_err());
        }

        #[test]
        fn 空や負数や非整数や数値範囲外を拒否する() {
            for input in [
                "",
                " ",
                "0",
                "-1",
                "1.5",
                "二",
                "99999999999999999999999999999",
            ] {
                assert!(TeamSetup::parse_count(input).is_err(), "{input}");
            }
        }

        #[test]
        fn 名前が人数分そろう前は完成できない() {
            let mut setup = TeamSetup::new(2).unwrap();
            setup.add_name("赤").unwrap();
            assert!(setup.finish().is_err());
        }

        #[test]
        fn 指定人数を超えて追加せず完成した一覧を保つ() {
            let mut setup = TeamSetup::new(1).unwrap();
            setup.add_name("赤").unwrap();
            assert!(setup.add_name("青").is_err());
            let teams = setup.finish().unwrap();
            assert_eq!(teams.len(), 1);
            assert_eq!(teams[0].name().as_str(), "赤");
        }
    }

    mod names {
        use super::*;

        #[test]
        fn 前後の空白を除き日本語と内部の空白と記号を保つ() {
            let mut setup = TeamSetup::new(1).unwrap();
            setup.add_name("　 赤 チーム! 🏆  ").unwrap();
            let teams = setup.finish().unwrap();
            assert_eq!(teams[0].name().as_str(), "赤 チーム! 🏆");
            assert_eq!(teams[0].score_board().total(), 0);
        }

        #[test]
        fn 空の名前を拒否して次の入力位置を進めない() {
            let mut setup = TeamSetup::new(1).unwrap();
            for name in ["", " ", "　"] {
                assert!(setup.add_name(name).is_err());
                assert_eq!(setup.next_number(), 1);
            }
            setup.add_name("赤").unwrap();
            assert_eq!(setup.finish().unwrap().len(), 1);
        }

        #[test]
        fn 日本語三十文字は許可し三十一文字を拒否する() {
            let mut setup = TeamSetup::new(1).unwrap();
            assert!(setup.add_name(&"赤".repeat(31)).is_err());
            setup.add_name(&"赤".repeat(30)).unwrap();
            assert_eq!(setup.finish().unwrap()[0].name().as_str(), "赤".repeat(30));
        }

        #[test]
        fn trimで消える前後の制御文字も拒否する() {
            let mut setup = TeamSetup::new(1).unwrap();
            for name in [
                "赤\n青",
                "\t赤",
                "赤\r",
                "赤\0",
                "赤\x1b[31m",
                "赤\u{7f}",
                "赤\u{85}",
            ] {
                assert!(setup.add_name(name).is_err());
                assert_eq!(setup.next_number(), 1);
            }
        }

        #[test]
        fn 同名でも別のチームを入力順に作る() {
            let names = vec![" 赤 ".to_owned(), "赤".to_owned()];
            let teams = TeamSetup::from_names(&names).unwrap();
            assert_eq!(teams[0].name(), teams[1].name());
            assert_ne!(teams[0].id(), teams[1].id());
        }
    }
}

mod throws {
    use super::*;

    mod valid {
        use super::*;

        #[test]
        fn 倒れた番号を空白区切りで変換し入力順に保持する() {
            let thrown = ThrowInput::parse(" 5  8 12 ").unwrap();
            assert_eq!(
                thrown
                    .skittles()
                    .iter()
                    .map(|pin| pin.number())
                    .collect::<Vec<_>>(),
                [5, 8, 12]
            );
        }

        #[test]
        fn 零は倒れた個体なしになる() {
            assert!(ThrowInput::parse(" 0 ").unwrap().skittles().is_empty());
        }

        #[test]
        fn 標準の十二本を変換できる() {
            let input = (1..=12)
                .map(|number| number.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(ThrowInput::parse(&input).unwrap().skittles().len(), 12);
        }
    }

    mod invalid {
        use super::*;

        #[test]
        fn 空と範囲外と数値以外と零の混在を拒否する() {
            for input in [
                "",
                " ",
                "x",
                "-1",
                "13",
                "1.5",
                "0 5",
                "5 0",
                "99999999999999",
            ] {
                assert!(ThrowInput::parse(input).is_err(), "{input}");
            }
        }

        #[test]
        fn 同じ番号を二度倒した入力を拒否する() {
            assert!(ThrowInput::parse("5 8 5").is_err());
        }
    }
}
