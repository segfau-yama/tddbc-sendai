use crate::Skittle;

pub struct CuiSkittle(Skittle);

impl From<Skittle> for CuiSkittle {
    fn from(value: Skittle) -> Self {
        Self(value)
    }
}

impl AsRef<Skittle> for CuiSkittle {
    fn as_ref(&self) -> &Skittle {
        &self.0
    }
}
