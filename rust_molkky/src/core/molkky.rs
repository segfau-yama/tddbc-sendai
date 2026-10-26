use super::{Field, Point, Position, Stick, Team};

pub struct Molkky {
    teams: Vec<Team>,
    field: Field,
    stick: Stick,
}

impl Molkky {
    pub fn new(teams: Vec<Team>) -> Self {
        Self::from_parts(teams, Field::default(), Stick::new(Position::new(0, 0)))
    }

    fn from_parts(teams: Vec<Team>, field: Field, stick: Stick) -> Self {
        assert!(!teams.is_empty(), "チームは1件以上必要です");
        for team in &teams {
            team.score_board().validate_for_game();
        }
        // 持ち込まれた得点がある場合も、勝者を一意に判定できる状態だけを受け付ける。
        assert!(
            teams
                .iter()
                .filter(|team| team.score_board().is_winner())
                .count()
                <= 1,
            "勝者が複数います"
        );
        Self {
            teams,
            field,
            stick,
        }
    }

    pub fn teams(&self) -> &[Team] {
        &self.teams
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    #[must_use]
    pub fn with_field(&self, field: Field) -> Self {
        Self {
            teams: self.teams.clone(),
            field,
            stick: self.stick,
        }
    }

    pub fn stick(&self) -> &Stick {
        &self.stick
    }

    #[must_use]
    pub fn with_stick(&self, stick: Stick) -> Self {
        Self {
            teams: self.teams.clone(),
            field: self.field.clone(),
            stick,
        }
    }

    /// 勝者がいればそのインデックスを返す。勝者なしの終了判定にはis_finishedを使う。
    pub fn result(&self) -> Option<usize> {
        if let Some(winner) = self
            .teams
            .iter()
            .position(|team| team.score_board().is_winner())
        {
            return Some(winner);
        }
        // 1チーム開始のゲームは、参加しただけでは勝利しない。
        if self.teams.len() < 2 {
            return None;
        }
        let mut remaining = self
            .teams
            .iter()
            .enumerate()
            .filter(|(_, team)| !team.score_board().is_disqualified());
        let (index, _) = remaining.next()?;
        remaining.next().is_none().then_some(index)
    }

    /// 勝者がいる場合、または全チームが失格の場合に終了とする。
    pub fn is_finished(&self) -> bool {
        self.result().is_some()
            || self
                .teams
                .iter()
                .all(|team| team.score_board().is_disqualified())
    }

    #[must_use]
    pub fn record_point(&self, team_index: usize, point: Point) -> Self {
        // 終了後は、得点や履歴を変更する前に拒否する。
        assert!(!self.is_finished(), "ゲームは終了しています");
        assert!(team_index < self.teams.len(), "チームが存在しません");
        assert!(
            !self.teams[team_index].score_board().is_disqualified(),
            "チームは失格しています"
        );
        let teams = self
            .teams
            .iter()
            .enumerate()
            .map(|(index, team)| {
                if index == team_index {
                    team.with_score_board(team.score_board().add(point))
                } else {
                    team.clone()
                }
            })
            .collect();
        Self::from_parts(teams, self.field.clone(), self.stick)
    }
}
