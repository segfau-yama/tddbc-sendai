use crate::{Skittle, SkittleState};

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

pub struct CuiSkittleState(SkittleState);

impl From<SkittleState> for CuiSkittleState {
    fn from(value: SkittleState) -> Self {
        Self(value)
    }
}

impl AsRef<SkittleState> for CuiSkittleState {
    fn as_ref(&self) -> &SkittleState {
        &self.0
    }
}
