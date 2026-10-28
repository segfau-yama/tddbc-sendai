use crate::core::models::{Skittle, SkittleState, Stick};
use rand::{Rng, RngCore};

/// 棒の識別子と、確率方式で使う設定を合成する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RandomStick {
    stick: Stick,
    fall_probability: f64,
}

impl Default for RandomStick {
    fn default() -> Self {
        Self::new(Stick::new(), 0.25)
    }
}

impl RandomStick {
    pub fn new(stick: Stick, fall_probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&fall_probability),
            "倒れる確率は0〜1です"
        );
        Self {
            stick,
            fall_probability,
        }
    }

    pub fn stick(&self) -> &Stick {
        &self.stick
    }

    pub fn fall_probability(&self) -> f64 {
        self.fall_probability
    }

    /// 全個体を返す。既に倒れている個体は抽選せず、倒れた状態を保つ。
    pub fn judge(&self, skittles: &[Skittle], rng: &mut dyn RngCore) -> Vec<Skittle> {
        skittles
            .iter()
            .map(|pin| {
                if pin.state() == SkittleState::Fall || rng.random_bool(self.fall_probability) {
                    pin.with_state(SkittleState::Fall)
                } else {
                    *pin
                }
            })
            .collect()
    }
}
