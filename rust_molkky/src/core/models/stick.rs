use super::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Stick {
    id: Id,
}

impl Stick {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> Id {
        self.id
    }
}

impl AsRef<Stick> for Stick {
    fn as_ref(&self) -> &Stick {
        self
    }
}

/// 投擲条件を更新した棒を返す。入力の意味と型は具象モデルが定める。
pub trait Throw<Input>: AsRef<Stick> + Sized {
    fn throw(&self, value: Input) -> Self;
}
