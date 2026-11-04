use crate::Skittle;

pub struct GuiSkittle(Skittle);

impl From<Skittle> for GuiSkittle {
    fn from(value: Skittle) -> Self {
        Self(value)
    }
}

impl AsRef<Skittle> for GuiSkittle {
    fn as_ref(&self) -> &Skittle {
        &self.0
    }
}
