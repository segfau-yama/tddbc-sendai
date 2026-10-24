use crate::{Team, Throw};

pub struct Molkky {
    teams: Vec<Team>,
}

impl Molkky {
    pub fn new(teams: Vec<Team>) -> Self {
        assert!(!teams.is_empty(), "チームは1件以上必要です");
        // 持ち込まれた得点がある場合も、勝者を一意に判定できる状態だけを受け付ける。
        assert!(
            teams
                .iter()
                .filter(|team| team.score().value() == 50)
                .count()
                <= 1,
            "勝者が複数います"
        );
        Self { teams }
    }

    pub fn teams(&self) -> &[Team] {
        &self.teams
    }

    /// 進行中ならNone、終了時は勝ったチームのインデックスを返す。
    pub fn result(&self) -> Option<usize> {
        // 超過はScore側で25点へ補正済み。50点ちょうどのチームだけが勝者になる。
        self.teams
            .iter()
            .position(|team| team.score().value() == 50)
    }

    pub fn record_throw(&mut self, team_index: usize, thrown: &Throw) -> i32 {
        // 終了後は、得点や履歴を変更する前に拒否する。
        assert!(self.result().is_none(), "ゲームは終了しています");
        self.teams
            .get_mut(team_index)
            .expect("チームが存在しません")
            .record_throw(thrown)
    }
}
