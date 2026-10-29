pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const ZERO_POINTS_LIMIT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point(i32);

impl Point {
    pub(crate) fn new(value: i32) -> Self {
        assert!(value >= 0, "点数は0以上です");
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreState {
    Calculated,
    Reset,
    Disqualified,
    Won,
}

#[derive(Clone)]
pub struct ScoreBoard {
    history: Vec<Point>,
    total: i32,
    state: ScoreState,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl ScoreBoard {
    pub fn new(history: Vec<Point>) -> Self {
        let (total, _, state) = history.iter().fold(
            (0, 0, ScoreState::Calculated),
            |(total, misses, state), point| {
                assert!(
                    !matches!(state, ScoreState::Won | ScoreState::Disqualified),
                    "終了後は投てきを記録できません"
                );
                let misses = if point.value() == 0 { misses + 1 } else { 0 };
                let total = total + point.value();
                let (total, state) = if misses == ZERO_POINTS_LIMIT {
                    (0, ScoreState::Disqualified)
                } else if total == TARGET_SCORE {
                    (total, ScoreState::Won)
                } else if total > TARGET_SCORE {
                    (RESET_SCORE, ScoreState::Reset)
                } else {
                    (total, ScoreState::Calculated)
                };
                (total, misses, state)
            },
        );
        Self {
            history,
            total,
            state,
        }
    }

    pub fn add(&self, point: Point) -> Self {
        assert!(
            !matches!(self.state, ScoreState::Won | ScoreState::Disqualified),
            "終了後は投てきを記録できません"
        );
        let mut history = self.history.clone();
        history.push(point);
        Self::new(history)
    }

    pub fn with_win(&self) -> Self {
        Self {
            total: TARGET_SCORE,
            state: ScoreState::Won,
            ..self.clone()
        }
    }

    pub fn total(&self) -> i32 {
        self.total
    }

    pub fn history(&self) -> Vec<Point> {
        self.history.clone()
    }

    pub fn state(&self) -> ScoreState {
        self.state
    }
}
