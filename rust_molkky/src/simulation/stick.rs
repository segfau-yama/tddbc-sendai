/// 確率方式の投てき条件。物理エンジンの棒の状態とは独立して扱う。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stick {
    fall_probability: f64,
}

impl Default for Stick {
    fn default() -> Self {
        Self::new(0.25)
    }
}

impl Stick {
    pub fn new(fall_probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&fall_probability),
            "倒れる確率は0〜1です"
        );
        Self { fall_probability }
    }

    pub fn fall_probability(&self) -> f64 {
        self.fall_probability
    }
}
