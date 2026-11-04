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

    fn hit(&self, stick: &CuiStick, skittle: &Self::Skittle) -> (Skittle, bool) {
        let fell = skittle.state() == SkittleState::Stand && rand::random_bool(stick.probability());
        if fell {
            (skittle.with_state(SkittleState::Fall), true)
        } else {
            (*skittle, false)
        }
    }
}
