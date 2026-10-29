use rust_molkky::Name;

mod storage {
    use super::*;

    mod whitespace {
        use super::*;

        #[test]
        fn 前後の空白も日本語と内部の空白と記号もそのまま保持する() {
            let input = "　 赤 チーム! 🏆  ";
            let name = Name::new(input);
            assert_eq!(name.as_str(), input);
            assert_eq!(name.to_string(), input);
        }

        #[test]
        fn 空文字と空白だけの文字列もそのまま保持する() {
            for input in ["", " ", "　"] {
                assert_eq!(Name::new(input).as_str(), input);
            }
        }
    }

    mod characters {
        use super::*;

        #[test]
        fn 三十一文字の名前もそのまま保持する() {
            let input = "赤".repeat(31);
            assert_eq!(Name::new(&input).as_str(), input);
        }

        #[test]
        fn 制御文字は前後にあってもそのまま保持する() {
            for input in [
                "赤\n青",
                "\t赤",
                "赤\r",
                "赤\0",
                "赤\x1b[31m",
                "赤\u{7f}",
                "赤\u{85}",
            ] {
                let name = Name::new(input);
                assert_eq!(name.as_str(), input);
                assert_eq!(name.to_string(), input);
            }
        }
    }
}

mod value_semantics {
    use super::*;

    #[test]
    fn 同じ文字列の名前と複製した名前は等しい() {
        let name = Name::new("赤チーム");
        assert_eq!(name, Name::new("赤チーム"));
        assert_eq!(name.clone(), name);
    }
}
