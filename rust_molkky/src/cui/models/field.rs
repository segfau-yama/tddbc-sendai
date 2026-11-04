use super::CuiSkittle;
use crate::{Field, Skittle};

pub struct CuiField {
    field: Field,
    pub skittles: Vec<CuiSkittle>,
}

impl CuiField {
    pub fn new(field: Field, skittles: Vec<CuiSkittle>) -> Self {
        Self { field, skittles }
    }
}

impl Default for CuiField {
    fn default() -> Self {
        Self::new(
            Field::new(),
            (1..=12).map(|number| Skittle::new(number).into()).collect(),
        )
    }
}

impl AsRef<Field> for CuiField {
    fn as_ref(&self) -> &Field {
        &self.field
    }
}
