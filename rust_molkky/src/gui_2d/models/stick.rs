use crate::core::Stick;

#[derive(Clone, Copy)]
pub struct GuiStick {
    stick: Stick,
    target: Option<[f32; 2]>,
    speed: f32,
}

impl Default for GuiStick {
    fn default() -> Self {
        Self::new(Stick::new(), None, 0.0)
    }
}

impl GuiStick {
    // 寸法はm、質量はkg。
    pub const RADIUS: f32 = 0.0275;
    pub const LENGTH: f32 = 0.225;
    pub const MASS: f32 = 0.25;

    /// targetはフィールド内のメートル座標、speedはm/s。入力検証はcontrollerで行う。
    pub fn new(stick: Stick, target: Option<[f32; 2]>, speed: f32) -> Self {
        Self {
            stick,
            target,
            speed,
        }
    }

    pub fn stick(&self) -> &Stick {
        &self.stick
    }

    pub fn target(&self) -> Option<[f32; 2]> {
        self.target
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }
}

impl AsRef<Stick> for GuiStick {
    fn as_ref(&self) -> &Stick {
        &self.stick
    }
}
