use crate::ScoreBoard;

#[derive(Clone)]
pub struct Team {
    name: String,
    score_board: ScoreBoard,
}

impl Team {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            score_board: ScoreBoard::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn score_board(&self) -> &ScoreBoard {
        &self.score_board
    }

    #[must_use]
    pub fn with_score_board(&self, score_board: ScoreBoard) -> Self {
        Self {
            name: self.name.clone(),
            score_board,
        }
    }
}
