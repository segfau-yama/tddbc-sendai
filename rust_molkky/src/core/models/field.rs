use super::{Skittle, validate_unique_numbers};

/// 全チーム共通のピン集合。表示位置は持たない。
#[derive(Clone)]
pub struct Field {
    skittles: Vec<Skittle>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new((1..=12).map(Skittle::new).collect())
    }
}

impl Field {
    pub fn new(skittles: Vec<Skittle>) -> Self {
        validate_unique_numbers(&skittles);
        Self { skittles }
    }
    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }
}
