pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const ZERO_POINTS_LIMIT: usize = 3;

/// 1回の投てきで得た非負の点数。
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
    total: i32,
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
        let mut total = 0;
        let mut misses = 0;
        let mut state = ScoreState::Calculated;
        for point in &history {
            assert!(
                !matches!(state, ScoreState::Won | ScoreState::Disqualified),
                "終了後の投てきは記録できません"
            );
            misses = if point.value() == 0 { misses + 1 } else { 0 };
            total += point.value();
            // 連続ミスによる失格は、それまでの得点にかかわらず0点にする。
            (total, state) = if misses == ZERO_POINTS_LIMIT {
                (0, ScoreState::Disqualified)
            } else if total == TARGET_SCORE {
                (total, ScoreState::Won)
            } else if total > TARGET_SCORE {
                (RESET_SCORE, ScoreState::Reset)
            } else {
                (total, ScoreState::Calculated)
            };
        }
        Self {
            history,
            total,
            state,
        }
    }

    /// 元の履歴を変更せず、新しい履歴を生成時と同じ経路で検証する。
    pub fn add(&self, point: Point) -> Self {
        assert!(
            !matches!(self.state, ScoreState::Won | ScoreState::Disqualified),
            "終了後の投てきは記録できません"
        );
        let mut history = self.history.clone();
        history.push(point);
        Self::new(history)
    }

    /// 他チームの失格による勝利。投てき履歴を増やさず50点にする。
    pub fn win_by_default(&self) -> Self {
        Self {
            total: TARGET_SCORE,
            state: ScoreState::Won,
            ..self.clone()
        }
    }

    /// 各投てきの補正と失格を反映した現在の得点。
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
