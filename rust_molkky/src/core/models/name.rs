use std::{error::Error, fmt};

/// 前後の空白を除去した1〜30 Unicodeスカラー値の名前。制御文字は許可しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong,
    ControlCharacter,
}

impl Name {
    pub fn new(value: &str) -> Result<Self, NameError> {
        // trimで消える改行・タブも、入力の制御文字として先に拒否する。
        if value.chars().any(char::is_control) {
            return Err(NameError::ControlCharacter);
        }
        let value = value.trim();
        if value.is_empty() {
            return Err(NameError::Empty);
        }
        if value.chars().count() > 30 {
            return Err(NameError::TooLong);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "チーム名は1文字以上必要です",
            Self::TooLong => "チーム名は30文字以内にしてください",
            Self::ControlCharacter => "チーム名に制御文字は使用できません",
        })
    }
}

impl Error for NameError {}
