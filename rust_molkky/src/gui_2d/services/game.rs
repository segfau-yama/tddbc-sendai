use super::{GuiMolkky, field::create_world};
use crate::core::{Molkky, SkittleState, StickState, Team, Throw};
use crate::gui_2d::models::{GuiField, GuiSkittle, GuiStick};

#[cfg(test)]
#[path = "../../../tests/gui_2d/services/game.rs"]
mod tests;

pub struct Game {
    teams: Vec<Team>,
    field: GuiField,
    stick: GuiStick,
    skittles: Vec<GuiSkittle>,
    rules: GuiMolkky,
}

impl Game {
    pub fn new(
        mut teams: Vec<Team>,
        field: GuiField,
        stick: GuiStick,
        skittles: Vec<GuiSkittle>,
    ) -> Self {
        for team in &mut teams {
            *team = team.with_turn(false);
        }
        let rules = create_world(&stick, &skittles);
        let mut game = Self {
            teams,
            field,
            stick,
            skittles,
            rules,
        };
        game.update_teams();
        game
    }

    pub fn teams(&self) -> &[Team] {
        &self.teams
    }
    pub fn field(&self) -> &GuiField {
        &self.field
    }
    pub fn stick(&self) -> &GuiStick {
        &self.stick
    }
    pub fn skittles(&self) -> &[GuiSkittle] {
        &self.skittles
    }
    pub fn winner(&self) -> Option<&Team> {
        self.rules.judge_game(&self.teams)
    }
    pub fn is_finished(&self) -> bool {
        self.rules.next_team(&self.teams).is_none()
    }
    pub fn is_simulating(&self) -> bool {
        self.stick.as_ref().state() != StickState::Held
    }

    pub fn aim(&mut self, target: [f32; 2], speed: f32) {
        if self.is_finished() || self.is_simulating() {
            return;
        }
        let prepared = GuiStick::new(*self.stick.as_ref(), Some(target), speed);
        (self.skittles, self.stick) = self.rules.update(&self.skittles, prepared);
    }

    pub fn throw(&mut self) -> bool {
        if self.is_finished() || self.is_simulating() {
            return false;
        }
        self.stick = GuiStick::throw(self.stick);
        if self.is_simulating() {
            true
        } else {
            self.cancel_aim();
            false
        }
    }

    pub fn cancel_aim(&mut self) {
        if self.is_simulating() {
            return;
        }
        self.stick = GuiStick::new(*self.stick.as_ref(), None, 0.0);
        self.skittles = self
            .skittles
            .iter()
            .map(|skittle| {
                skittle.update(
                    skittle.position(),
                    skittle.angle(),
                    skittle.as_ref().state(),
                    false,
                )
            })
            .collect();
    }

    pub fn update(&mut self) {
        if self.is_finished() || !self.is_simulating() {
            return;
        }
        (self.skittles, self.stick) = self.rules.update(&self.skittles, self.stick);
        // 棒だけが停止しても、スキットルの連鎖が終わるまでは採点しない。
        if self.stick.as_ref().state() == StickState::Stop
            && self
                .skittles
                .iter()
                .all(|skittle| skittle.as_ref().state() != SkittleState::Hit)
        {
            self.finish_throw();
        }
    }

    fn finish_throw(&mut self) {
        if !self.is_simulating() {
            return;
        }
        let fallen: Vec<_> = self
            .skittles
            .iter()
            .filter(|skittle| skittle.as_ref().state() == SkittleState::Fall)
            .map(|skittle| *skittle.as_ref())
            .collect();
        let current = self
            .teams
            .iter()
            .position(Team::turn)
            .expect("投擲中は手番があります");
        let team = &self.teams[current];
        self.teams[current] =
            team.with_score_board(team.score_board().add(self.rules.calc_score(&fallen)));
        self.stick = GuiStick::new(*self.stick.as_ref(), None, 0.0);
        self.update_teams();
        if !self.is_finished() {
            self.skittles = self
                .skittles
                .iter()
                .map(|skittle| self.rules.stand_up(skittle))
                .collect();
        }
    }

    fn update_teams(&mut self) {
        let winner = self.rules.judge_game(&self.teams).map(Team::id);
        let next = self.rules.next_team(&self.teams).map(Team::id);
        for team in &mut self.teams {
            *team = team.with_turn(next == Some(team.id()));
            if winner == Some(team.id()) {
                *team = team.with_score_board(team.score_board().with_win());
            }
        }
    }
}
