use super::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StickState {
    #[default]
    Held,
    Move,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Stick {
    id: Id,
    state: StickState,
}

impl Stick {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn state(&self) -> StickState {
        self.state
    }

    pub fn with_state(&self, state: StickState) -> Self {
        Self { state, ..*self }
    }
}

impl AsRef<Stick> for Stick {
    fn as_ref(&self) -> &Stick {
        self
    }
}

pub trait Throw: AsRef<Stick> + Sized {
    /// 個体自身の投擲条件から、投げた状態の新しい個体を返す。
    fn throw(stick: Self) -> Self;
}

pub trait Move: AsRef<Stick> + Sized {
    /// 個体自身の運動結果から、移動・停止の状態を反映した個体を返す。
    fn r#move(stick: Self) -> Self;
}
