use crate::{Skittle, skittle::validate_unique_numbers};

/// 1回の投てきの点数。calc_scoreが0〜12の範囲を保証する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point(i32);

impl Point {
    pub fn value(&self) -> i32 {
        self.0
    }
}

/// 倒れたピンだけから採点する。入力・履歴・乱数源は変更しない。
pub fn calc_score(knocked_over_skittles: &[Skittle]) -> Point {
    validate_unique_numbers(knocked_over_skittles);
    // 1本なら番号が点数になり、0本・複数本なら本数が点数になる。
    Point(match knocked_over_skittles {
        [skittle] => skittle.number(),
        skittles => skittles.len() as i32,
    })
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
        self.score_history.iter().fold(0, |total, point| {
            let next = total + point.value();
            // 超過した回だけ25点に戻し、その後の投てきは25点から加算する。
            if next > 50 { 25 } else { next }
        })
    }

    pub fn history(&self) -> &[Point] {
        &self.score_history
    }
}
