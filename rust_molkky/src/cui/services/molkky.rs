use crate::cui::models::CuiStick;
use crate::{Molkky, Skittle, SkittleState};

#[derive(Default)]
pub struct CuiMolkky {
    numbers: Vec<i32>,
}

impl CuiMolkky {
    pub fn new(numbers: Vec<i32>) -> Self {
        Self { numbers }
    }
}

impl Molkky for CuiMolkky {
    type Stick = CuiStick;
    type Skittle = Skittle;
    type ThrowOutput = f64;

    fn aim(&self, _stick: &CuiStick, skittles: &[Self::Skittle]) -> Vec<Skittle> {
        self.numbers
            .iter()
            .filter_map(|number| skittles.iter().find(|skittle| skittle.number() == *number))
            .copied()
            .collect()
    }

    fn throw(&mut self, stick: &CuiStick) -> f64 {
        stick.probability()
    }

    fn hit(&self, stick: &CuiStick, skittles: &[Self::Skittle]) -> Vec<Skittle> {
        let probability = stick.probability();
        skittles
            .iter()
            .filter(|_| rand::random_bool(probability))
            .map(|skittle| skittle.with_state(SkittleState::Fall))
            .collect()
    }
}
