use crate::core::models::{Point, Skittle, validate_unique_ids};

/// 位置や投てき方法に依存せず、倒れた番号だけから採点する。
pub fn calc_score(knocked_over_skittles: &[Skittle]) -> Point {
    validate_unique_ids(knocked_over_skittles);
    Point::new(match knocked_over_skittles {
        [skittle] => skittle.number(),
        skittles => i32::try_from(skittles.len()).expect("スキットル本数が得点の範囲外です"),
    })
}
