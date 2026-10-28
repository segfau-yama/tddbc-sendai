use rust_molkky::{Name, NameError};

#[test]
fn 前後の空白を除去し日本語と内部の空白と記号を保持する() {
    let name = Name::new("　 赤 チーム! 🏆  ").unwrap();
    assert_eq!(name.as_str(), "赤 チーム! 🏆");
    assert_eq!(name.to_string(), "赤 チーム! 🏆");
    assert_eq!(name.clone(), name);
}

#[test]
fn 名前の長さはバイト数ではなく一から三十のunicode文字で検証する() {
    assert!(Name::new("赤").is_ok());
    assert!(Name::new(&"赤".repeat(30)).is_ok());
    assert_eq!(Name::new(&"赤".repeat(31)), Err(NameError::TooLong));
    for invalid in ["", " ", "　"] {
        assert_eq!(Name::new(invalid), Err(NameError::Empty));
    }
}

#[test]
fn 制御文字は前後にあっても拒否する() {
    for invalid in [
        "赤\n青",
        "\t赤",
        "赤\r",
        "赤\0",
        "赤\x1b[31m",
        "赤\u{7f}",
        "赤\u{85}",
    ] {
        assert_eq!(Name::new(invalid), Err(NameError::ControlCharacter));
    }
}
