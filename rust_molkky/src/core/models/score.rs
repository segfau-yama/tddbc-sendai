pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const CONSECUTIVE_ZERO_LIMIT: usize = 3;

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
        let (_, _, state, reset_at) = history.iter().enumerate().fold(
            (0_i64, 0, ScoreState::Calculated, None),
            |(total, consecutive_zeros, state, reset_at), (index, point)| {
                assert!(state != ScoreState::Won, "50点到達後に記録があります");
                assert!(state != ScoreState::Disqualified, "失格後に記録があります");
                let total = total + i64::from(point.value());
                let consecutive_zeros = if point.value() == 0 {
                    consecutive_zeros + 1
                } else {
                    0
                };
                let state = if consecutive_zeros >= CONSECUTIVE_ZERO_LIMIT {
                    ScoreState::Disqualified
                } else if total == i64::from(TARGET_SCORE) {
                    ScoreState::Won
                } else if total > i64::from(TARGET_SCORE) {
                    ScoreState::Reset
                } else {
                    ScoreState::Calculated
                };
                let (total, reset_at) = if state == ScoreState::Reset {
                    (i64::from(RESET_SCORE), Some(index))
                } else {
                    (total, reset_at)
                };
                (total, consecutive_zeros, state, reset_at)
            },
        );
        // 超過前の履歴を25点に置き換え、その後の投てきだけを残す。
        let history = if let Some(index) = reset_at {
            std::iter::once(Point::new(RESET_SCORE))
                .chain(history.into_iter().skip(index + 1))
                .collect()
        } else {
            history
        };
        Self { history, state }
    }

    /// 元の履歴を変更せず、新しい履歴を生成時と同じ経路で検証する。
    pub fn add(&self, point: Point) -> Self {
        let mut history = self.history.clone();
        history.push(point);
        Self::new(history)
    }

    /// 直近の超過補正で[25]へ置き換えた計算用履歴。全投てきの記録ではない。
    pub fn history(&self) -> &[Point] {
        &self.history
    }

    /// 生成時に補正した履歴から合計を算出する。
    pub fn total(&self) -> i32 {
        self.history.iter().map(Point::value).sum()
    }

    pub fn state(&self) -> ScoreState {
        self.state
    }
}
