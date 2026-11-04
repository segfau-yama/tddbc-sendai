use super::{
    controller::{Controller, PointerSample, timestamp},
    models::{GuiField, GuiStick},
    services::{GuiMolkky, field::create_field, teams::from_names},
    view,
};
use crate::core::{Molkky, Skittle, SkittleState, Team};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::time::Duration;

#[cfg(test)]
#[path = "../../tests/gui_2d/app.rs"]
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
enum Phase {
    Ready,
    Charging,
    Simulating,
}

/// GUIの進行と入力・物理・coreの接続を所有する。
struct AppState {
    teams: Vec<Team>,
    field: GuiField,
    stick: GuiStick,
    phase: Phase,
    rules: GuiMolkky,
    input: Controller,
    last_frame: Duration,
    accumulator: Duration,
}

impl AppState {
    fn new(mut teams: Vec<Team>) -> Self {
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
            input: Controller::default(),
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

    fn handle(&mut self, event: DomEvent) {
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
                    let stick = GuiStick::new(*self.stick.as_ref(), Some(target), speed);
                    self.stick = stick;
                    if self.rules.throw(&stick) {
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
                        let step = Duration::from_secs_f32(GuiMolkky::TIME_STEP);
                        while self.accumulator >= step && !self.rules.is_settled() {
                            self.rules.step();
                            self.accumulator -= step;
                        }
                        // 接触はstepが蓄積する。既にFallの個体も保持し、描画はこの状態だけを読む。
                        let updated = self
                            .field
                            .skittles()
                            .iter()
                            .map(|skittle| self.rules.hit(&self.stick, skittle).0)
                            .collect();
                        self.field = GuiField::new(*self.field.as_ref(), updated);
                        if self.rules.is_settled() {
                            let fallen: Vec<_> = self
                                .field
                                .skittles()
                                .iter()
                                .filter(|skittle| skittle.state() == SkittleState::Fall)
                                .copied()
                                .collect();
                            self.finish_throw(&fallen);
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
            self.rules.stand_up();
            let standing = self
                .field
                .skittles()
                .iter()
                .map(|skittle| skittle.with_state(SkittleState::Stand))
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

fn render(state: &AppState) -> Element {
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
    let (position, angle) = if input_phase == Phase::Simulating {
        rules.pose(stick.stick().id()).unwrap()
    } else {
        (GuiField::THROW_POSITION, 0.0)
    };
    let targets = if input_phase == Phase::Charging {
        rules.aim(stick, field.skittles())
    } else {
        Vec::new()
    };
    let skittles = field.skittles().iter().map(|skittle| {
        let pos = rules.pose(skittle.id()).unwrap().0;
        let id = skittle.id();
        let class = if targets.iter().any(|target| target.id() == id) {
            "aimed"
        } else {
            ""
        };
        rsx! { g { key: "{id}", class, {view::skittle(skittle, pos, GuiMolkky::SKITTLE_RADIUS)} } }
    });
    let scene = rsx! {
        {skittles}
        {view::aim(GuiField::THROW_POSITION, stick.target())}
        {view::stick(stick.stick().id(), position, angle, GuiMolkky::STICK_LENGTH, 2.0 * GuiMolkky::STICK_RADIUS)}
    };
    rsx! {
        style { {include_str!("view/style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {view::field(GuiField::BOUNDS, scene)}
                {view::speedometer(stick.speed(), Controller::MAX_SPEED)}
            }
            div { class: "scores",
                {view::scores(teams)}
                if finished { {view::result(rules.judge_game(teams))} }
            }
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| AppState::new(from_names(&["赤チーム", "青チーム"])));
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let content = render(&state.read());
    rsx! {
        main {
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("app.js"));
                bridge.set(Some(eval));
                spawn(async move {
                    while let Ok(event) = eval.recv::<DomEvent>().await { state.write().handle(event); }
                    state.write().handle(DomEvent::Blur);
                    error.set(true);
                });
            },
            document::Title { "モルック" }
            {content}
            if error() { p { role: "alert", "入力の接続が停止しました。画面を再読み込みしてください。" } }
        }
    }
}
