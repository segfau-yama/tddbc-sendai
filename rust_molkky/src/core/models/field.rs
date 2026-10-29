use super::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Field {
    id: Id,
}

impl Field {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> Id {
        self.id
    }
}
