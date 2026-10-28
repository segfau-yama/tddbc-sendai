use super::Id;

/// Standは直立、Fallは転倒済みを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkittleState {
    Stand,
    Fall,
}

/// 番号は得点用。同じ番号でも、Idが異なれば別の個体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Skittle {
    id: Id,
    number: i32,
    state: SkittleState,
}

impl Skittle {
    /// 1〜12以外の番号はpanicで拒否する。番号にNoneは指定できない。
    pub fn new(number: i32) -> Self {
        // 存在するスキットルの番号を生成時に保証する。
        assert!(
            (1..=12).contains(&number),
            "スキットル番号は1〜12です: {number}"
        );
        Self {
            id: Id::new(),
            number,
            state: SkittleState::Stand,
        }
    }

    pub fn state(&self) -> SkittleState {
        self.state
    }

    pub fn with_state(&self, state: SkittleState) -> Self {
        Self { state, ..*self }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn number(&self) -> i32 {
        self.number
    }
}
