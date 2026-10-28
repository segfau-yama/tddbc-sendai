use super::{Point, Skittle};
use std::collections::HashSet;

/// 個体一覧の読み取り専用モデル。所有や採点対象の選択は呼び出し側が行う。
pub struct Skittles<'a> {
    skittles: &'a [Skittle],
}

impl<'a> Skittles<'a> {
    pub fn new(skittles: &'a [Skittle]) -> Self {
        Self { skittles }
    }

    /// 同番号の別個体は許可し、同じIDの二重登録を拒否する。
    pub fn validate(&self) {
        let mut ids = HashSet::new();
        for skittle in self.skittles {
            assert!(
                ids.insert(skittle.id()),
                "スキットルIDが重複しています: {}",
                skittle.id()
            );
        }
    }

    /// この一覧を採点する。今回倒れた個体の選別は呼び出し側で行う。
    pub fn calc_score(&self) -> Point {
        self.validate();
        Point::new(match self.skittles {
            [skittle] => skittle.number(),
            skittles => i32::try_from(skittles.len()).expect("スキットル本数が得点の範囲外です"),
        })
    }
}
