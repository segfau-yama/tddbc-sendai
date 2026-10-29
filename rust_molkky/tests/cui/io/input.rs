use rust_molkky::cui::io::{Input, Output};
use rust_molkky::{Name, Team};
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};

mod teams {
    use super::*;

    #[test]
    fn チーム数と名前を不正な入力の後に再入力できる() {
        let mut reader = Cursor::new("x\n0\n2\n\n 赤 \n青\n");
        let mut output = Vec::new();
        let teams = read_teams(&mut reader, &mut output, &[]).unwrap().unwrap();
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
        let teams = read_teams(&mut Cursor::new("2\r\n赤\r\n青\r\n"), &mut Vec::new(), &[])
            .unwrap()
            .unwrap();
        assert_eq!(teams[0].name().as_str(), "赤");
        assert_eq!(teams[1].name().as_str(), "青");
    }

    #[test]
    fn 引数の名前を検証し標準入力のチーム設定を省略する() {
        let mut reader = Cursor::new("q\n");
        let names = vec![" 赤 ".to_owned(), "青".to_owned()];
        let teams = read_teams(&mut reader, &mut Vec::new(), &names)
            .unwrap()
            .unwrap();
        assert_eq!(teams[0].name().as_str(), "赤");
        assert_eq!(teams.len(), 2);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn 引数の不正な名前は入力エラーを返す() {
        let names = vec!["赤\n青".to_owned()];
        let error = read_teams(&mut Cursor::new(""), &mut Vec::new(), &names)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn 設定途中の入力終了やqでは未完成のチームを返さない() {
        for text in ["", "2\n赤\n", "q\n", "1\nq\n"] {
            assert!(
                read_teams(&mut Cursor::new(text), &mut Vec::new(), &[])
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
        let thrown = read_throw(&mut Cursor::new("\n13\n5 5\n5 8\n"), &mut output, &team)
            .unwrap()
            .unwrap();
        assert_eq!(thrown, [5, 8]);
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
                read_throw(&mut Cursor::new(text), &mut Vec::new(), &team)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn 零をミスとして読み取る() {
        let team = Team::new(Name::new("赤"));
        let thrown = read_throw(&mut Cursor::new("0\n"), &mut Vec::new(), &team)
            .unwrap()
            .unwrap();
        assert!(thrown.is_empty());
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
        let error = read_teams(&mut reader, &mut Vec::new(), &[]).err().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::Other);
    }
}

fn read_teams(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    names: &[String],
) -> io::Result<Option<Vec<Team>>> {
    Input::new(reader)
        .teams(&mut Output::new(writer), names)
        .map(|result| result.map(|teams| teams.teams))
}

fn read_throw(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    team: &Team,
) -> io::Result<Option<Vec<i32>>> {
    Input::new(reader).throw_result(&mut Output::new(writer), team)
}

mod validation {
    use super::*;

    mod count {
        use super::*;

        #[test]
        fn 空や負数や非整数や数値範囲外を拒否して再入力する() {
            for invalid in [
                "",
                " ",
                "0",
                "-1",
                "1.5",
                "二",
                "99999999999999999999999999999",
            ] {
                let mut buffer = Vec::new();
                let teams = read_teams(
                    &mut Cursor::new(format!("{invalid}\n 1 \n赤\n")),
                    &mut buffer,
                    &[],
                )
                .unwrap()
                .unwrap();
                assert_eq!(teams.len(), 1);
                assert_eq!(
                    String::from_utf8(buffer)
                        .unwrap()
                        .matches("入力エラー")
                        .count(),
                    1,
                    "{invalid}"
                );
            }
        }

        #[test]
        fn 指定した人数分だけ名前を読み次の投てき入力を残す() {
            let script = "2\n赤\n青\n5\n";
            let mut reader = Cursor::new(script);
            let teams = read_teams(&mut reader, &mut Vec::new(), &[])
                .unwrap()
                .unwrap();
            assert_eq!(teams.len(), 2);
            assert_eq!(&script[reader.position() as usize..], "5\n");
        }
    }

    mod numbers {
        use super::*;

        #[test]
        fn 空白区切りの番号を入力順に変換する() {
            let team = Team::new(Name::new("赤"));
            let numbers = read_throw(&mut Cursor::new(" 12  5 8 \n"), &mut Vec::new(), &team)
                .unwrap()
                .unwrap();
            assert_eq!(numbers, [12, 5, 8]);
        }

        #[test]
        fn 標準の十二本を入力できる() {
            let script = (1..=12)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let team = Team::new(Name::new("赤"));
            let numbers = read_throw(&mut Cursor::new(script), &mut Vec::new(), &team)
                .unwrap()
                .unwrap();
            assert_eq!(numbers, (1..=12).collect::<Vec<_>>());
        }

        #[test]
        fn 空と範囲外と非整数と零の混在と番号重複を拒否する() {
            let team = Team::new(Name::new("赤"));
            for invalid in [
                "",
                " ",
                "x",
                "-1",
                "13",
                "1.5",
                "0 5",
                "5 0",
                "99999999999999",
                "5 8 5",
            ] {
                let mut buffer = Vec::new();
                let numbers = read_throw(
                    &mut Cursor::new(format!("{invalid}\n0\n")),
                    &mut buffer,
                    &team,
                )
                .unwrap()
                .unwrap();
                assert!(numbers.is_empty());
                assert_eq!(
                    String::from_utf8(buffer)
                        .unwrap()
                        .matches("入力エラー")
                        .count(),
                    1,
                    "{invalid}"
                );
            }
        }
    }
}
