use super::{Field, GameStatus, Point, Stick, Team};
use crate::core::services::judge_game;

pub struct Molkky {
    teams: Vec<Team>,
    field: Field,
    stick: Stick,
}

impl Molkky {
    pub fn new(teams: Vec<Team>) -> Self {
        Self::from_parts(teams, Field::default(), Stick::default())
    }

    fn from_parts(teams: Vec<Team>, field: Field, stick: Stick) -> Self {
        judge_game(&teams);
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
        match self.status() {
            GameStatus::Won { team_index } => Some(team_index),
            _ => None,
        }
    }

    pub fn status(&self) -> GameStatus {
        judge_game(&self.teams)
    }

    pub fn is_finished(&self) -> bool {
        !matches!(self.status(), GameStatus::InProgress)
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
