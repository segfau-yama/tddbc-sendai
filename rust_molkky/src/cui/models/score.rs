use crate::{Point, ScoreBoard, ScoreState};

pub struct CuiPoint(Point);

impl From<Point> for CuiPoint {
    fn from(value: Point) -> Self {
        Self(value)
    }
}

impl AsRef<Point> for CuiPoint {
    fn as_ref(&self) -> &Point {
        &self.0
    }
}

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

pub struct CuiScoreState(ScoreState);

impl From<ScoreState> for CuiScoreState {
    fn from(value: ScoreState) -> Self {
        Self(value)
    }
}

impl AsRef<ScoreState> for CuiScoreState {
    fn as_ref(&self) -> &ScoreState {
        &self.0
    }
}
