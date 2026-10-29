use crate::{Molkky, Skittle, SkittleState, Stick};

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
    fn aim(&self, _stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle> {
        self.numbers
            .iter()
            .filter_map(|number| skittles.iter().find(|pin| pin.number() == *number))
            .copied()
            .collect()
    }

    fn hit(&self, _stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle> {
        skittles
            .iter()
            .map(|pin| pin.with_state(SkittleState::Fall))
            .collect()
    }
}
