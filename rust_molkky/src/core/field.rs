use super::{Position, Skittle, skittle::validate_unique_numbers};

#[derive(Clone)]
pub struct Field {
    skittles: Vec<Skittle>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new(
            (1..=12)
                .map(|number| {
                    Skittle::new(number)
                        .with_position(Position::new((number - 1) % 3 - 1, (number - 1) / 3 + 3))
                })
                .collect(),
        )
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
