use super::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkittleState {
    Stand,
    Fall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Skittle {
    id: Id,
    number: i32,
    state: SkittleState,
}

impl Skittle {
    pub fn new(number: i32) -> Self {
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
