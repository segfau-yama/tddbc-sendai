use crate::{Field, Skittle};

pub struct CuiField {
    field: Field,
    pub skittles: Vec<Skittle>,
}

impl CuiField {
    pub fn new(field: Field, skittles: Vec<Skittle>) -> Self {
        Self { field, skittles }
    }
}

impl Default for CuiField {
    fn default() -> Self {
        Self::new(Field::new(), (1..=12).map(Skittle::new).collect())
    }
}

impl AsRef<Field> for CuiField {
    fn as_ref(&self) -> &Field {
        &self.field
    }
}
