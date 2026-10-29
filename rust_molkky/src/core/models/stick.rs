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
