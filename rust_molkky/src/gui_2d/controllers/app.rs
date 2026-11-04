use super::{
    field,
    mouse::{MouseInput, PointerSample, timestamp},
};
use crate::core::{Molkky, Skittle, SkittleState, Team, Throw};
use crate::gui_2d::{
    models::{GuiField, GuiStick},
    services::{GuiMolkky, TIME_STEP, field::create_field},
    view,
};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::time::Duration;

#[cfg(test)]
#[path = "../../../tests/gui_2d/controllers/app.rs"]
mod tests;

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DomEvent {
    Press(PointerSample),
    Move(PointerSample),
    Release(PointerSample),
    Cancel { pointer_id: i32 },
    Blur,
    Frame { time: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Ready,
    Charging,
    Simulating,
}

/// GUIの進行と入力・物理・coreの接続を所有する。
pub(crate) struct Controller {
    teams: Vec<Team>,
    field: GuiField,
    stick: GuiStick,
    phase: Phase,
    rules: GuiMolkky,
    input: MouseInput,
    last_frame: Duration,
    accumulator: Duration,
}

impl Controller {
    pub(crate) fn new(mut teams: Vec<Team>) -> Self {
        for team in &mut teams {
            *team = team.with_turn(false);
        }
        let (field, stick, rules) = create_field();
        let mut state = Self {
            teams,
            field,
            stick,
            phase: Phase::Ready,
            rules,
            input: MouseInput::default(),
            last_frame: Duration::ZERO,
            accumulator: Duration::ZERO,
        };
        state.update_game_state();
        state
    }

    fn is_finished(&self) -> bool {
        self.rules.next_team(&self.teams).is_none()
    }

    const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

    pub(crate) fn handle(&mut self, event: DomEvent) {
        match event {
            DomEvent::Press(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                if self.phase == Phase::Ready
                    && !self.is_finished()
                    && self.input.press(
                        sample.pointer_id,
                        Some(MouseButton::from_web_code(sample.button)),
                        sample.target(),
                        now,
                    )
                {
                    let (target, speed) = self.input.preview(now).unwrap();
                    self.stick = GuiStick::new(*self.stick.as_ref(), Some(target), speed);
                    self.phase = Phase::Charging;
                }
            }
            DomEvent::Move(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.input.move_to(sample.pointer_id, sample.target());
                if let Some((target, speed)) = self.input.preview(now) {
                    self.stick = GuiStick::new(*self.stick.as_ref(), Some(target), speed);
                }
            }
            DomEvent::Release(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.input.move_to(sample.pointer_id, sample.target());
                if let Some((target, speed)) = self.input.release(
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    now,
                ) {
                    let aimed =
                        GuiStick::new(*self.stick.as_ref(), Some(target), self.stick.speed());
                    self.stick = aimed.throw(speed);
                    if target != GuiField::THROW_POSITION && speed > 0.0 {
                        self.phase = Phase::Simulating;
                        self.last_frame = now;
                        self.accumulator = Duration::ZERO;
                    } else {
                        self.reset_input();
                        self.phase = Phase::Ready;
                    }
                }
            }
            DomEvent::Cancel { pointer_id } => {
                self.input.cancel(Some(pointer_id));
                if self.input.pointer_id().is_none() && self.phase == Phase::Charging {
                    self.reset_input();
                    self.phase = Phase::Ready;
                }
            }
            DomEvent::Blur => {
                self.input.cancel(None);
                if self.phase == Phase::Charging {
                    self.reset_input();
                    self.phase = Phase::Ready;
                }
            }
            DomEvent::Frame { time } => {
                if let Some(now) = timestamp(time) {
                    if self.phase == Phase::Charging {
                        if let Some((target, speed)) = self.input.preview(now) {
                            self.stick = GuiStick::new(*self.stick.as_ref(), Some(target), speed);
                        }
                    } else if self.phase == Phase::Simulating {
                        // タブ復帰時に大量の物理更新で描画を止めない。
                        self.accumulator += now
                            .saturating_sub(self.last_frame)
                            .min(Self::MAX_FRAME_TIME);
                        self.last_frame = now.max(self.last_frame);
                        let step = Duration::from_secs_f32(TIME_STEP);
                        while self.accumulator >= step {
                            let flying = self.rules.fly(&self.stick);
                            self.accumulator -= step;
                            // 最後の飛行更新の接触も採点前に反映し、既にFallの個体も保持する。
                            let updated = self
                                .field
                                .skittles()
                                .iter()
                                .map(|skittle| self.rules.hit(&self.stick, skittle))
                                .collect();
                            self.field = GuiField::new(*self.field.as_ref(), updated);
                            if !flying {
                                let fallen: Vec<_> = self
                                    .field
                                    .skittles()
                                    .iter()
                                    .filter(|skittle| skittle.state() == SkittleState::Fall)
                                    .copied()
                                    .collect();
                                self.finish_throw(&fallen);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn reset_input(&mut self) {
        self.input.cancel(None);
        self.stick = GuiStick::new(*self.stick.stick(), None, 0.0);
    }

    fn finish_throw(&mut self, fallen: &[Skittle]) {
        if self.phase != Phase::Simulating {
            return;
        }
        let current = self
            .teams
            .iter()
            .position(Team::turn)
            .expect("投擲中は手番があります");
        let team = &self.teams[current];
        self.teams[current] =
            team.with_score_board(team.score_board().add(self.rules.calc_score(fallen)));
        self.reset_input();
        self.update_game_state();
        if !self.is_finished() {
            let standing = self
                .field
                .skittles()
                .iter()
                .map(|skittle| self.rules.stand_up(skittle))
                .collect();
            self.field = GuiField::new(*self.field.as_ref(), standing);
        }
    }

    fn update_game_state(&mut self) {
        let winner = self.rules.judge_game(&self.teams).map(Team::id);
        let next = self.rules.next_team(&self.teams).map(Team::id);
        for team in &mut self.teams {
            *team = team.with_turn(next == Some(team.id()));
            if winner == Some(team.id()) {
                *team = team.with_score_board(team.score_board().with_win());
            }
        }
        self.phase = Phase::Ready;
    }
}

pub(crate) fn render(state: &Controller) -> Element {
    let teams = &state.teams;
    let field = &state.field;
    let stick = &state.stick;
    let finished = state.is_finished();
    let input_phase = state.phase;
    let rules = &state.rules;
    let (phase, status) = if finished {
        ("finished", "試合終了")
    } else {
        match input_phase {
            Phase::Ready => ("ready", "左ボタンを押して照準・蓄力を開始します。"),
            Phase::Charging => ("charging", "マウスで狙い、左ボタンを離すと投擲します。"),
            Phase::Simulating => ("simulating", "投擲中…"),
        }
    };
    rsx! {
        style { {include_str!("../view/style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {field::render(field, stick, rules, input_phase)}
                {view::speedometer(stick.speed(), MouseInput::MAX_SPEED)}
            }
            div { class: "scores",
                {view::scores(teams)}
                if finished { {view::result(rules.judge_game(teams))} }
            }
        }
    }
}
