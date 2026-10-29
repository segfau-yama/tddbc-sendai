use crate::Skittle;

pub struct Field {
    pub skittles: Vec<Skittle>,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            skittles: (1..=12).map(Skittle::new).collect(),
        }
    }
}
