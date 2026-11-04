use crate::ScoreBoard;

pub struct CuiScoreBoard(ScoreBoard);

impl From<ScoreBoard> for CuiScoreBoard {
    fn from(value: ScoreBoard) -> Self {
        Self(value)
    }
}

impl AsRef<ScoreBoard> for CuiScoreBoard {
    fn as_ref(&self) -> &ScoreBoard {
        &self.0
    }
}
