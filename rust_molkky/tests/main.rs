use std::process::Command;

#[test]
fn 不正なチーム名は端末初期化前に入力エラーとして報告する() {
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
