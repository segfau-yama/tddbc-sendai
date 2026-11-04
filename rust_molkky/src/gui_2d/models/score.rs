use crate::ScoreBoard;

pub struct GuiScoreBoard(ScoreBoard);

impl From<ScoreBoard> for GuiScoreBoard {
    fn from(value: ScoreBoard) -> Self {
        Self(value)
    }
}

impl AsRef<ScoreBoard> for GuiScoreBoard {
    fn as_ref(&self) -> &ScoreBoard {
        &self.0
    }
}
