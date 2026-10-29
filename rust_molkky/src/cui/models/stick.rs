use crate::Stick;

pub struct CuiStick {
    stick: Stick,
    probability: f64,
}

impl CuiStick {
    pub fn new(stick: Stick, probability: f64) -> Self {
        assert!((0.0..=1.0).contains(&probability), "確率は0以上1以下です");
        Self { stick, probability }
    }

    pub fn stick(&self) -> &Stick {
        &self.stick
    }

    pub fn probability(&self) -> f64 {
        self.probability
    }
}

impl Default for CuiStick {
    fn default() -> Self {
        Self::new(Stick::new(), 0.5)
    }
}
