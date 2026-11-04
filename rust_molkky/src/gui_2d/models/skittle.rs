use crate::Skittle;

pub struct GuiSkittle(Skittle);

impl GuiSkittle {
    // 半径はm、質量はkg。
    pub const RADIUS: f32 = 0.0275;
    pub const MASS: f32 = 0.15;
}

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
