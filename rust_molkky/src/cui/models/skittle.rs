use crate::{Skittle, StandUp};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CuiSkittle {
    skittle: Skittle,
    probability: f64,
}

impl CuiSkittle {
    /// 転倒後に次の個体へ連鎖する確率。初回転倒ではStickの命中確率を保持する。
    pub fn new(skittle: Skittle, probability: f64) -> Self {
        assert!((0.0..=1.0).contains(&probability), "確率は0以上1以下です");
        Self {
            skittle,
            probability,
        }
    }

    pub fn probability(&self) -> f64 {
        self.probability
    }
}

impl From<Skittle> for CuiSkittle {
    fn from(skittle: Skittle) -> Self {
        Self::new(skittle, 0.0)
    }
}

impl AsRef<Skittle> for CuiSkittle {
    fn as_ref(&self) -> &Skittle {
        &self.skittle
    }
}

impl StandUp for CuiSkittle {
    fn stand_up(self) -> Self {
        self.skittle.stand_up().into()
    }
}
