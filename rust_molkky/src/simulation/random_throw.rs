use super::Stick;
use crate::core::{Field, ThrowOutcome, ThrowStrategy};
use rand::Rng;
use std::convert::Infallible;

/// 各個体を独立抽選する方式。Field内の座標には依存しない。
pub struct RandomThrow<'a, R: ?Sized> {
    stick: &'a Stick,
    rng: &'a mut R,
}
impl<'a, R: Rng + ?Sized> RandomThrow<'a, R> {
    pub fn new(stick: &'a Stick, rng: &'a mut R) -> Self {
        Self { stick, rng }
    }
}
impl<F: Field + Clone, R: Rng + ?Sized> ThrowStrategy<F> for RandomThrow<'_, R> {
    type Error = Infallible;
    fn throw(&mut self, field: &F) -> Result<ThrowOutcome<F>, Infallible> {
        let knocked_over = field
            .skittles()
            .iter()
            .copied()
            .filter(|_| self.rng.random_bool(self.stick.fall_probability()))
            .collect();
        Ok(ThrowOutcome::new(field.clone(), knocked_over))
    }
}
