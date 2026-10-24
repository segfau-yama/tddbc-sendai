use std::collections::HashMap;

use crate::Skittle;

pub struct Throw {
    skittles: HashMap<i32, Skittle>,
}

impl Throw {
    /// 同じ番号の重複をpanicで拒否し、入力のコピーを保持する。
    /// スライスやその要素にNoneは指定できない。
    pub fn new(skittles: &[Skittle]) -> Self {
        let mut validated = HashMap::new();
        for &skittle in skittles {
            // 同じ番号のスキットルは1本だけなので、重複を黙って除去せず拒否する。
            assert!(
                validated.insert(skittle.number(), skittle).is_none(),
                "スキットル番号が重複しています: {}",
                skittle.number()
            );
        }
        Self {
            skittles: validated,
        }
    }

    pub fn calc_point(&self) -> i32 {
        // 1本なら番号が点数になり、0本・複数本なら本数が点数になる。
        if self.skittles.len() == 1 {
            return self
                .skittles
                .values()
                .next()
                .expect("1本のスキットルが存在する")
                .number();
        }
        self.skittles.len() as i32
    }
}
