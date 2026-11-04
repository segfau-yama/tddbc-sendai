use crate::cui::models::{CuiSkittle, CuiStick};
use crate::{Molkky, SkittleState};

#[cfg(test)]
#[path = "../../../tests/cui/services/molkky.rs"]
mod tests;

pub struct CuiMolkky {
    numbers: Vec<i32>,
    random: fn() -> f64,
}

impl Default for CuiMolkky {
    fn default() -> Self {
        Self {
            numbers: Vec::new(),
            random: rand::random::<f64>,
        }
    }
}

impl CuiMolkky {
    pub const CHAIN_DECAY_PERCENT: f64 = 20.0;

    pub fn new(numbers: Vec<i32>) -> Self {
        Self {
            numbers,
            ..Self::default()
        }
    }
}

impl Molkky for CuiMolkky {
    type Stick = CuiStick;
    type Skittle = CuiSkittle;
    type ThrowOutput = f64;

    fn aim(&self, _stick: &CuiStick, skittles: &[Self::Skittle]) -> Vec<Self::Skittle> {
        self.numbers
            .iter()
            .filter_map(|number| skittles.iter().find(|s| s.as_ref().number() == *number))
            .copied()
            .collect()
    }

    fn throw(&mut self, stick: &CuiStick) -> f64 {
        stick.probability()
    }

    fn hit(&self, stick: &CuiStick, skittle: &Self::Skittle) -> Self::Skittle {
        if skittle.as_ref().state() == SkittleState::Stand && (self.random)() < stick.probability()
        {
            CuiSkittle::new(
                skittle.as_ref().with_state(SkittleState::Fall),
                stick.probability(),
            )
        } else {
            *skittle
        }
    }

    fn chain_hit(
        &self,
        fallen_skittle: &Self::Skittle,
        chained_skittle: &Self::Skittle,
    ) -> Self::Skittle {
        if fallen_skittle.as_ref().state() != SkittleState::Fall
            || chained_skittle.as_ref().state() == SkittleState::Fall
        {
            return *chained_skittle;
        }
        // 80→60→40→20→0で丸め残差を残さないため、パーセント単位で減衰させる。
        let probability =
            ((fallen_skittle.probability() * 100.0 - Self::CHAIN_DECAY_PERCENT) / 100.0).max(0.0);
        if (self.random)() < probability {
            CuiSkittle::new(
                chained_skittle.as_ref().with_state(SkittleState::Fall),
                probability,
            )
        } else {
            *chained_skittle
        }
    }
}
