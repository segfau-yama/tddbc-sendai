use super::Id;

/// 表現やシミュレーション設定から独立した個体の識別子。
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
