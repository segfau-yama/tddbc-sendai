use crate::{Point, Team};

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
                .filter(|team| team.score_board().value() == 50)
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
        // 超過はScoreBoard側で25点へ補正済み。50点ちょうどのチームだけが勝者になる。
        self.teams
            .iter()
            .position(|team| team.score_board().value() == 50)
    }

    #[must_use]
    pub fn record_point(&self, team_index: usize, point: Point) -> Self {
        // 終了後は、得点や履歴を変更する前に拒否する。
        assert!(self.result().is_none(), "ゲームは終了しています");
        let team = self.teams.get(team_index).expect("チームが存在しません");
        let mut teams = self.teams.clone();
        teams[team_index] = team.record_point(point);
        Self { teams }
    }
}
