pub const TARGET_SCORE: i32 = 50;
pub const RESET_SCORE: i32 = 25;
pub const CONSECUTIVE_ZERO_LIMIT: usize = 3;

fn apply_point(total: i32, point: Point) -> (i32, bool) {
    let next = total + point.value();
    let reset = next > TARGET_SCORE;
    (if reset { RESET_SCORE } else { next }, reset)
}

/// 1回の投てきの点数。calc_scoreが0〜12の範囲を保証する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point(i32);

impl Point {
    pub(crate) fn new(value: i32) -> Self {
        assert!((0..=12).contains(&value), "点数は0〜12です");
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct ScoreBoard {
    score_history: Vec<Point>,
}

impl ScoreBoard {
    pub fn new() -> Self {
        Self::default()
    }

    /// 元の履歴を変更せず、点数を追加した新しい得点表を返す。
    #[must_use]
    pub fn add(&self, point: Point) -> Self {
        let mut score_history = self.score_history.clone();
        score_history.push(point);
        Self { score_history }
    }

    pub fn value(&self) -> i32 {
        self.evaluate_total().0
    }

    pub fn is_winner(&self) -> bool {
        self.value() == TARGET_SCORE
    }

    pub fn is_disqualified(&self) -> bool {
        self.score_history
            .windows(CONSECUTIVE_ZERO_LIMIT)
            .any(|points| points.iter().all(|point| point.value() == 0))
    }

    /// 直近の投てきで超過補正が適用されたかを返す。
    pub fn was_reset(&self) -> bool {
        self.evaluate_total().1
    }

    fn evaluate_total(&self) -> (i32, bool) {
        self.score_history
            .iter()
            .fold((0, false), |(total, _), &point| apply_point(total, point))
    }

    pub(crate) fn validate_for_game(&self) {
        let mut total = 0;
        let mut consecutive_zeros = 0;
        for &point in &self.score_history {
            // 最終得点だけでなく、終了後に記録された履歴も構築時に拒否する。
            assert!(total != TARGET_SCORE, "50点到達後に記録があります");
            assert!(
                consecutive_zeros < CONSECUTIVE_ZERO_LIMIT,
                "失格後に記録があります"
            );
            total = apply_point(total, point).0;
            consecutive_zeros = if point.value() == 0 {
                consecutive_zeros + 1
            } else {
                0
            };
        }
        assert!(
            (0..=TARGET_SCORE).contains(&total),
            "現在の得点が範囲外です"
        );
    }

    pub fn history(&self) -> &[Point] {
        &self.score_history
    }
}
