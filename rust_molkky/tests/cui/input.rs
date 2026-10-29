use rust_molkky::cui::input;
use rust_molkky::{Name, Team};
use std::io::{self, BufReader, Cursor, Read};

mod teams {
    use super::*;

    #[test]
    fn チーム数と名前を不正な入力の後に再入力できる() {
        let mut reader = Cursor::new("x\n0\n2\n\n 赤 \n青\n");
        let mut output = Vec::new();
        let teams = input::teams(&mut reader, &mut output, &[])
            .unwrap()
            .unwrap();
        assert_eq!(
            teams
                .iter()
                .map(|team| team.name().as_str())
                .collect::<Vec<_>>(),
            ["赤", "青"]
        );
        assert_eq!(
            String::from_utf8(output)
                .unwrap()
                .matches("入力エラー")
                .count(),
            3
        );
    }

    #[test]
    fn crlfを名前の制御文字と誤認せず行末として読み取る() {
        let teams = input::teams(&mut Cursor::new("2\r\n赤\r\n青\r\n"), &mut Vec::new(), &[])
            .unwrap()
            .unwrap();
        assert_eq!(teams[0].name().as_str(), "赤");
        assert_eq!(teams[1].name().as_str(), "青");
    }

    #[test]
    fn 引数の名前を検証し標準入力のチーム設定を省略する() {
        let mut reader = Cursor::new("q\n");
        let names = vec![" 赤 ".to_owned(), "青".to_owned()];
        let teams = input::teams(&mut reader, &mut Vec::new(), &names)
            .unwrap()
            .unwrap();
        assert_eq!(teams[0].name().as_str(), "赤");
        assert_eq!(teams.len(), 2);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn 引数の不正な名前は入力エラーを返す() {
        let names = vec!["赤\n青".to_owned()];
        let error = input::teams(&mut Cursor::new(""), &mut Vec::new(), &names)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn 設定途中の入力終了やqでは未完成のチームを返さない() {
        for text in ["", "2\n赤\n", "q\n", "1\nq\n"] {
            assert!(
                input::teams(&mut Cursor::new(text), &mut Vec::new(), &[])
                    .unwrap()
                    .is_none()
            );
        }
    }
}

mod throws {
    use super::*;

    #[test]
    fn 不正な投てきは再入力し検証できた番号だけを返す() {
        let team = Team::new(Name::new("赤"));
        let mut output = Vec::new();
        let thrown = input::throw_result(&mut Cursor::new("\n13\n5 5\n5 8\n"), &mut output, &team)
            .unwrap()
            .unwrap();
        assert_eq!(
            thrown
                .skittles()
                .iter()
                .map(|pin| pin.number())
                .collect::<Vec<_>>(),
            [5, 8]
        );
        assert_eq!(
            String::from_utf8(output)
                .unwrap()
                .matches("入力エラー")
                .count(),
            3
        );
        assert_eq!(team.score_board().history().len(), 0);
    }

    #[test]
    fn qと入力終了で投てきを作らず終了する() {
        let team = Team::new(Name::new("赤"));
        for text in ["", "q\n", " Q \n"] {
            assert!(
                input::throw_result(&mut Cursor::new(text), &mut Vec::new(), &team)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn 零をミスとして読み取る() {
        let team = Team::new(Name::new("赤"));
        let thrown = input::throw_result(&mut Cursor::new("0\n"), &mut Vec::new(), &team)
            .unwrap()
            .unwrap();
        assert!(thrown.skittles().is_empty());
    }
}

mod errors {
    use super::*;

    struct FailedReader;
    impl Read for FailedReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::Other.into())
        }
    }

    #[test]
    fn 入力のioエラーを再入力で握りつぶさない() {
        let mut reader = BufReader::new(FailedReader);
        let error = input::teams(&mut reader, &mut Vec::new(), &[])
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::Other);
    }
}
