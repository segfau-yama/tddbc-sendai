use super::{Id, Name, ScoreBoard};

#[derive(Clone)]
pub struct Team {
    id: Id,
    name: Name,
    score_board: ScoreBoard,
}

impl Team {
    pub fn new(name: Name) -> Self {
        Self {
            id: Id::new(),
            name,
            score_board: ScoreBoard::default(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn score_board(&self) -> &ScoreBoard {
        &self.score_board
    }

    pub fn with_score_board(&self, score_board: ScoreBoard) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            score_board,
        }
    }
}
