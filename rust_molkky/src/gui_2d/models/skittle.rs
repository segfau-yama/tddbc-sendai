use crate::{Skittle, SkittleState, StandUp};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiSkittle {
    skittle: Skittle,
    position: [f32; 2],
    angle: f32,
    aimed: bool,
}

impl GuiSkittle {
    // 寸法はm、質量はkg。
    pub const RADIUS: f32 = 0.0275;
    pub const MASS: f32 = 0.15;

    pub fn new(skittle: Skittle, position: [f32; 2]) -> Self {
        Self {
            skittle,
            position,
            angle: 0.0,
            aimed: false,
        }
    }
    pub fn position(&self) -> [f32; 2] {
        self.position
    }
    pub fn angle(&self) -> f32 {
        self.angle
    }
    pub fn aimed(&self) -> bool {
        self.aimed
    }

    pub fn update(self, position: [f32; 2], angle: f32, state: SkittleState, aimed: bool) -> Self {
        Self {
            skittle: self.skittle.with_state(state),
            position,
            angle,
            aimed,
        }
    }
}

impl From<Skittle> for GuiSkittle {
    fn from(skittle: Skittle) -> Self {
        Self::new(skittle, [0.0; 2])
    }
}

impl AsRef<Skittle> for GuiSkittle {
    fn as_ref(&self) -> &Skittle {
        &self.skittle
    }
}

impl StandUp for GuiSkittle {
    fn stand_up(self) -> Self {
        Self::new(self.skittle.stand_up(), self.position)
    }
}
