use crate::Throw;

pub struct Score {
    history: Vec<i32>,
}

impl Score {
    /// 投てき順に点数を記録する。スライスやその要素にNoneは指定できない。
    pub fn new(recorded_throws: &[&Throw]) -> Self {
        Self {
            history: recorded_throws
                .iter()
                .map(|thrown| thrown.calc_point())
                .collect(),
        }
    }

    pub fn value(&self) -> i32 {
        self.history.iter().fold(0, |total, point| {
            let next = total + point;
            // 超過した回だけ25点に戻し、その後の投てきは25点から加算する。
            if next > 50 { 25 } else { next }
        })
    }

    pub fn history(&self) -> &[i32] {
        &self.history
    }

    pub fn with_throw(&self, thrown: &Throw) -> Self {
        let mut history = self.history.clone();
        history.push(thrown.calc_point());
        Self { history }
    }
}
