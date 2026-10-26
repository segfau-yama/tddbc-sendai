use std::collections::HashSet;

pub(crate) fn validate_unique_numbers(skittles: &[Skittle]) {
    let mut numbers = HashSet::new();
    for skittle in skittles {
        // ピンは番号で識別する。同じ番号は重複になる。
        assert!(
            numbers.insert(skittle.number()),
            "スキットル番号が重複しています: {}",
            skittle.number()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Skittle {
    number: i32,
}

impl Skittle {
    /// 1〜12以外の番号はpanicで拒否する。番号にNoneは指定できない。
    pub fn new(number: i32) -> Self {
        // 存在するスキットルの番号を生成時に保証する。
        assert!(
            (1..=12).contains(&number),
            "スキットル番号は1〜12です: {number}"
        );
        Self { number }
    }

    pub fn number(&self) -> i32 {
        self.number
    }
}
