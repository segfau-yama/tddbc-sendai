use super::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stick {
    position: Position,
}

impl Stick {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> Position {
        self.position
    }
}
