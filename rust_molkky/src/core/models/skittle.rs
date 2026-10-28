use super::Id;
use std::collections::HashSet;

pub(crate) fn validate_unique_ids(skittles: &[Skittle]) {
    let mut ids = HashSet::new();
    for skittle in skittles {
        assert!(
            ids.insert(skittle.id()),
            "スキットルIDが重複しています: {}",
            skittle.id()
        );
    }
}

/// 番号は得点用。同じ番号でも、Idが異なれば別の個体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Skittle {
    id: Id,
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
        Self {
            id: Id::new(),
            number,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn number(&self) -> i32 {
        self.number
    }
}
