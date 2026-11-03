use super::{
    controller::{Controller, DomEvent, timestamp},
    models::{Game, GamePhase, GuiField, GuiStick},
    services::GuiMolkky,
    view,
};
use crate::core::{Molkky, Name, Skittle, SkittleState, Team};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::time::Duration;

#[cfg(test)]
#[path = "../../tests/gui_2d/app.rs"]
mod tests;

struct AppState {
    game: Game,
    rules: GuiMolkky,
    input: Controller,
    last_frame: Duration,
    accumulator: Duration,
}

impl AppState {
    const ORIGIN: [f32; 2] = [0.0, 0.0];
    const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

    fn new(teams: Vec<Team>) -> Self {
        let mut game = Game::new(teams);
        for team in &mut game.teams {
            *team = team.with_turn(false);
        }
        // 手前からの標準番号配列。円同士が接する三角格子に配置する。
        let rows: &[&[i32]] = &[&[1, 2], &[3, 10, 4], &[5, 11, 12, 6], &[7, 9, 8]];
        let radius = GuiMolkky::SKITTLE_RADIUS;
        let placements: Vec<_> = rows
            .iter()
            .enumerate()
            .flat_map(|(row, numbers)| {
                let skittles = game.field.skittles();
                numbers.iter().enumerate().map(move |(column, number)| {
                    let skittle = skittles
                        .iter()
                        .find(|skittle| skittle.number() == *number)
                        .unwrap();
                    let x = (2.0 * column as f32 - (numbers.len() - 1) as f32) * radius;
                    (
                        skittle.id(),
                        [x, 1.0 + row as f32 * 3.0_f32.sqrt() * radius],
                    )
                })
            })
            .collect();
        let rules = GuiMolkky::new(&game.stick, Self::ORIGIN, &placements);
        let mut state = Self {
            game,
            rules,
            input: Controller::default(),
            last_frame: Duration::ZERO,
            accumulator: Duration::ZERO,
        };
        state.advance_turn();
        state
    }

    fn handle(&mut self, event: DomEvent) {
        match event {
            DomEvent::Press(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                if self.input.press(
                    self.game.phase,
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    sample.target(),
                    now,
                ) {
                    self.game.stick = self.input.preview(&self.game.stick, now).unwrap();
                    self.game.phase = GamePhase::Charging;
                }
            }
            DomEvent::Move(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.input.move_to(sample.pointer_id, sample.target());
                if let Some(stick) = self.input.preview(&self.game.stick, now) {
                    self.game.stick = stick;
                }
            }
            DomEvent::Release(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.input.move_to(sample.pointer_id, sample.target());
                if let Some(stick) = self.input.release(
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    &self.game.stick,
                    now,
                ) {
                    self.game.stick = stick;
                    if self.rules.throw(&stick) {
                        self.game.phase = GamePhase::Simulating;
                        self.last_frame = now;
                        self.accumulator = Duration::ZERO;
                    } else {
                        self.reset_aim();
                        self.game.phase = GamePhase::Ready;
                    }
                }
            }
            DomEvent::Cancel { pointer_id } => {
                self.input.cancel_pointer(pointer_id);
                self.cancel_charge();
            }
            DomEvent::Blur => {
                self.input.cancel();
                self.cancel_charge();
            }
            DomEvent::Frame { time } => {
                if let Some(now) = timestamp(time) {
                    self.update(now);
                }
            }
        }
    }

    fn cancel_charge(&mut self) {
        if self.game.phase == GamePhase::Charging && self.input.pointer_id().is_none() {
            self.reset_aim();
            self.game.phase = GamePhase::Ready;
        }
    }

    fn reset_aim(&mut self) {
        self.game.stick = GuiStick::new(*self.game.stick.stick(), None, 0.0);
    }

    fn update(&mut self, now: Duration) {
        if self.game.phase == GamePhase::Charging {
            if let Some(stick) = self.input.preview(&self.game.stick, now) {
                self.game.stick = stick;
            }
        } else if self.game.phase == GamePhase::Simulating {
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
            if self.rules.is_settled() {
                let fallen = self.rules.hit(&self.game.stick, self.game.field.skittles());
                self.finish_throw(&fallen);
            }
        }
    }

    fn finish_throw(&mut self, fallen: &[Skittle]) {
        if self.game.phase != GamePhase::Simulating {
            return;
        }
        let current = self
            .game
            .teams
            .iter()
            .position(Team::turn)
            .expect("投擲中は手番があります");
        let team = &self.game.teams[current];
        self.game.teams[current] =
            team.with_score_board(team.score_board().add(self.rules.calc_score(fallen)));
        self.reset_aim();
        self.advance_turn();
    }

    fn advance_turn(&mut self) {
        let winner = self.rules.judge_game(&self.game.teams).map(Team::id);
        let next = if winner.is_some() {
            None
        } else {
            self.rules.next_team(&self.game.teams).map(Team::id)
        };
        for team in &mut self.game.teams {
            *team = team.with_turn(next == Some(team.id()));
            if winner == Some(team.id()) {
                *team = team.with_score_board(team.score_board().with_win());
            }
        }
        self.game.phase = if next.is_some() {
            GamePhase::Ready
        } else {
            GamePhase::Finished
        };
    }

    fn render(&self) -> Element {
        let phase = match self.game.phase {
            GamePhase::Ready => "ready",
            GamePhase::Charging => "charging",
            GamePhase::Simulating => "simulating",
            GamePhase::Finished => "finished",
        };
        let status = match self.game.phase {
            GamePhase::Ready => "左ボタンを押して照準・蓄力を開始します。",
            GamePhase::Charging => "マウスで狙い、左ボタンを離すと投擲します。",
            GamePhase::Simulating => "投擲中…",
            GamePhase::Finished => "試合終了",
        };
        let (position, angle) = if self.game.phase == GamePhase::Simulating {
            self.rules.pose(self.game.stick.stick().id()).unwrap()
        } else {
            (Self::ORIGIN, 0.0)
        };
        let fallen = self.rules.hit(&self.game.stick, self.game.field.skittles());
        let targets = if self.game.phase == GamePhase::Charging {
            self.rules.aim(&self.game.stick, self.game.field.skittles())
        } else {
            Vec::new()
        };
        let skittles = self.game.field.skittles().iter().map(|skittle| {
            let pos = self.rules.pose(skittle.id()).unwrap().0;
            let skittle = if matches!(self.game.phase, GamePhase::Simulating | GamePhase::Finished)
                && fallen.iter().any(|fallen_skittle| fallen_skittle.id() == skittle.id())
            {
                skittle.with_state(SkittleState::Fall)
            } else {
                *skittle
            };
            let id = skittle.id();
            let class = if targets.iter().any(|target| target.id() == id) {
                "aimed"
            } else {
                ""
            };
            rsx! { g { key: "{id}", class, {view::skittle(&skittle, pos, GuiMolkky::SKITTLE_RADIUS)} } }
        });
        let scene = rsx! {
            {skittles}
            {view::aim(Self::ORIGIN, self.game.stick.target())}
            {view::stick(self.game.stick.stick().id(), position, angle, GuiMolkky::STICK_LENGTH, 2.0 * GuiMolkky::STICK_RADIUS)}
        };
        rsx! {
            style { {include_str!("view/style.css")} }
            h1 { "モルック" }
            p { role: "status", "{status}" }
            div { class: "game-layout",
                div { id: "molkky-input", "data-phase": phase,
                    {view::field(GuiField::BOUNDS, scene)}
                    {view::speedometer(self.game.stick.speed(), Controller::MAX_SPEED)}
                }
                div { class: "scores",
                    {view::scores(&self.game.teams)}
                    if self.game.phase == GamePhase::Finished { {view::result(self.rules.judge_game(&self.game.teams))} }
                }
            }
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| {
        AppState::new(vec![
            Team::new(Name::new("赤チーム")),
            Team::new(Name::new("青チーム")),
        ])
    });
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let content = state.read().render();
    rsx! {
        main {
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("controller.js"));
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
