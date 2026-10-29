use std::vec;

pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const ZERO_POINTS_LIMIT: usize = 3;

/// 非負の点数。投てきの得点に加え、履歴中では超過補正後の25点も表す。
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

/// 直近の計算状態。WonとDisqualifiedに達した後は履歴を追加できない。
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
    state: ScoreState,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl ScoreBoard {
    /// 履歴の途中で終了した後の追加記録も、生成時に拒否する。
    pub fn new(history: Vec<Point>) -> Self {
        let total: i32 = history.iter().map(|point| point.value()).sum();
        let zero_points = history.iter().filter(|point| point.value() == 0).count();
        if total == TARGET_SCORE {
            return Self {
                history: history,
                state: ScoreState::Won,
            };
        }
        if total > TARGET_SCORE {
            return Self {
                history: vec![Point::new(RESET_SCORE)],
                state: ScoreState::Reset,
            };
        }
        if zero_points >= ZERO_POINTS_LIMIT {
            return Self {
                history: history,
                state: ScoreState::Disqualified,
            };
        }
        return Self {
            history: history,
            state: ScoreState::Calculated,
        };
    }

    /// 元の履歴を変更せず、新しい履歴を生成時と同じ経路で検証する。
    pub fn add(&self, point: Point) -> Self {
        let mut history = self.history.clone();
        history.push(point);
        Self::new(history)
    }

    /// 生成時に補正した履歴から合計を算出する。
    pub fn total(&self) -> i32 {
        self.history.iter().map(Point::value).sum()
    }

    pub fn history(&self) -> Vec<Point> {
        self.history.clone()
    }

    pub fn state(&self) -> ScoreState {
        self.state
    }
}
