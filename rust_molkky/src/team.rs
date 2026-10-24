use crate::{Score, Throw};

pub struct Team {
    name: String,
    score: Score,
}

impl Team {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            score: Score::new(&[]),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn record_throw(&mut self, thrown: &Throw) -> i32 {
        self.score = self.score.with_throw(thrown);
        thrown.calc_point()
    }
}
