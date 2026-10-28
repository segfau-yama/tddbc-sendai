pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const CONSECUTIVE_ZERO_LIMIT: usize = 3;

fn apply_point(total: i32, point: Point) -> (i32, bool) {
    let next = i64::from(total) + i64::from(point.value());
    let reset = ScoreBoard::was_reset(next);
    (if reset { RESET_SCORE } else { next as i32 }, reset)
}

/// 1回の投てきの非負の点数。変則構成では倒れた本数が12を超えることもある。
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
    score_history: Vec<Point>,
    state: ScoreState,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl ScoreBoard {
    /// 履歴の途中で終了した後の追加記録も、生成時に拒否する。
    pub fn new(score_history: Vec<Point>) -> Self {
        let mut total = 0;
        let mut consecutive_zeros = 0;
        let mut state = ScoreState::Calculated;
        for &point in &score_history {
            assert!(state != ScoreState::Won, "50点到達後に記録があります");
            assert!(state != ScoreState::Disqualified, "失格後に記録があります");
            let (next, reset) = apply_point(total, point);
            total = next;
            consecutive_zeros = if point.value() == 0 {
                consecutive_zeros + 1
            } else {
                0
            };
            state = if Self::is_disqualified(consecutive_zeros) {
                ScoreState::Disqualified
            } else if Self::is_winner(total) {
                ScoreState::Won
            } else if reset {
                ScoreState::Reset
            } else {
                ScoreState::Calculated
            };
        }
        Self {
            score_history,
            state,
        }
    }

    /// 元の履歴を変更せず、新しい履歴を生成時と同じ経路で検証する。
    pub fn add(&self, point: Point) -> Self {
        let mut history = self.score_history.clone();
        history.push(point);
        Self::new(history)
    }

    pub fn history(&self) -> &[Point] {
        &self.score_history
    }

    /// 各投てきの25点補正を反映して履歴から合計を算出する。
    pub fn total(&self) -> i32 {
        self.score_history
            .iter()
            .fold(0, |total, &point| apply_point(total, point).0)
    }

    pub fn state(&self) -> ScoreState {
        self.state
    }

    fn is_winner(total: i32) -> bool {
        total == TARGET_SCORE
    }
    fn is_disqualified(consecutive_zeros: usize) -> bool {
        consecutive_zeros >= CONSECUTIVE_ZERO_LIMIT
    }
    fn was_reset(total: i64) -> bool {
        total > i64::from(TARGET_SCORE)
    }
}
