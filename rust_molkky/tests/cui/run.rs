use rust_molkky::cui::run as game;
use rust_molkky::{Stick, cui::models::CuiStick};
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};

fn play(script: &str, names: &[&str]) -> String {
    let names = names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    game::run(
        &mut Cursor::new(script),
        &mut output,
        &names,
        &CuiStick::new(Stick::new(), 1.0),
    )
    .unwrap();
    String::from_utf8(output).unwrap()
}

mod game_loop {
    use super::*;

    #[test]
    fn チーム設定の後に交互に投げて得点を記録する() {
        let text = play("2\n 赤 \n青\n5\n8\nq\n", &[]);
        assert!(text.contains("赤: 5点 (1投)"));
        assert!(text.contains("青: 8点 (1投)"));
        assert!(text.contains("終了しました"));
    }

    #[test]
    fn 五十点で勝利した後の入力を記録しない() {
        let script = "12\n1\n12\n1\n12\n1\n12\n1\n2\n12\n";
        let names = vec!["赤".to_owned(), "青".to_owned()];
        let mut reader = Cursor::new(script);
        let mut output = Vec::new();
        game::run(
            &mut reader,
            &mut output,
            &names,
            &CuiStick::new(Stick::new(), 1.0),
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("赤の勝利です（50点）"));
        assert!(text.contains("赤: 50点 (5投)"));
        assert!(text.contains("青: 4点 (4投)"));
        assert!(reader.position() < script.len() as u64);
        assert!(!text.rsplit("得点表").next().unwrap().contains('▶'));
    }

    #[test]
    fn 最後の生存者は投てきを増やさず五十点で勝つ() {
        let text = play("0\n5\n0\n5\n0\n", &["赤", "青"]);
        assert!(text.contains("赤: 0点 (3投) [失格]"));
        assert!(text.contains("青: 50点 (2投) [勝利]"));
        assert!(text.contains("青の勝利です（50点）"));
    }

    #[test]
    fn 単独参加の失格は勝者なしで終了する() {
        let text = play("5\n0\n0\n0\n12\n", &["赤"]);
        assert!(text.contains("赤: 0点 (4投) [失格]"));
        assert!(text.contains("全チームが失格"));
        assert!(!text.contains("勝利です"));
    }

    #[test]
    fn 三チームでは失格者を飛ばして残りのチームが交代する() {
        let text = play("0\n5\n8\n0\n5\n8\n0\n5\n8\nq\n", &["赤", "青", "緑"]);
        assert!(text.contains("赤: 0点 (3投) [失格]"));
        assert!(text.contains("青: 15点 (3投)"));
        assert!(text.contains("緑: 24点 (3投)"));
    }

    #[test]
    fn 連続しないミスを累計で失格にしない() {
        let text = play("0\n0\n1\n0\n0\n5\nq\n", &["赤"]);
        assert!(text.contains("赤: 6点 (6投)"));
        assert!(!text.contains("[失格]"));
    }

    #[test]
    fn 超過補正の前後で投数と得点を保つ() {
        let text = play("12\n12\n12\n12\n3\n10\nq\n", &["赤"]);
        assert!(text.contains("赤: 25点 (5投) [25点に補正]"));
        assert!(text.contains("赤: 35点 (6投)"));
    }

    #[test]
    fn 不正な投てきでは投数も手番も進まない() {
        let text = play("13\n5 5\n5 8\nq\n", &["赤", "青"]);
        assert_eq!(text.matches("入力エラー").count(), 2);
        assert!(text.contains("赤: 2点 (1投)"));
        assert!(text.contains("▶ 2. 青: 0点 (0投)"));
    }

    #[test]
    fn 設定途中の入力終了とqで終了できる() {
        for script in ["", "q\n", "2\n赤\n"] {
            let text = play(script, &[]);
            assert!(text.contains("終了しました"));
            assert!(!text.contains("得点表"));
        }
    }
}

mod cli {
    use super::*;

    #[test]
    fn 不正な引数のチーム名はゲーム開始前に入力エラーとして報告する() {
        for (input, expected) in [
            ("　".to_owned(), "チーム名は1文字以上必要です"),
            ("赤".repeat(31), "チーム名は30文字以内にしてください"),
            ("赤\n青".to_owned(), "チーム名に制御文字は使用できません"),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_rust_molkky"))
                .arg(input)
                .output()
                .unwrap();
            assert!(!output.status.success());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains(expected), "{stderr}");
            assert!(!stderr.contains("panicked"));
            assert!(!stderr.contains("対話型ターミナル"));
        }
    }

    fn execute(script: &str, names: &[&str]) -> std::process::Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rust_molkky"))
            .args(names)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    #[test]
    fn 実バイナリを標準入力で操作できる() {
        let output = execute("2\n赤\n青\n5 8\nq\n", &[]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(
            [0, 2, 5, 8]
                .iter()
                .any(|score| text.contains(&format!("赤: {score}点 (1投)")))
        );
        assert!(text.contains("終了しました"));
    }

    #[test]
    fn 引数のチーム名でも起動できる() {
        let output = execute("q\n", &["赤", "青"]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("赤: 0点 (0投)"));
        assert!(!text.contains("チーム数"));
    }

    #[test]
    fn 設定途中の標準入力終了で停止する() {
        let output = execute("2\n赤\n", &[]);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("終了しました")
        );
    }
}

mod probability {
    use super::*;

    #[test]
    fn 確率零では狙った番号を得点にせず手番を進める() {
        let names = vec!["赤".to_owned(), "青".to_owned()];
        let mut output = Vec::new();
        game::run(
            &mut Cursor::new("5\n8\nq\n"),
            &mut output,
            &names,
            &CuiStick::new(Stick::new(), 0.0),
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("赤: 0点 (1投)"));
        assert!(text.contains("青: 0点 (1投)"));
        assert!(text.contains("終了しました"));
    }
}
