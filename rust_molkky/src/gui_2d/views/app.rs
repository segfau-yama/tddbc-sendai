use super::{
    mouse::{MouseInput, PointerSample, timestamp},
    *,
};
use crate::core::Team;
use crate::gui_2d::{
    models::GuiField,
    services::{Game, TIME_STEP, field::create_field, teams::from_names},
};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::time::Duration;

#[cfg(test)]
#[path = "../../../tests/gui_2d/views/app.rs"]
mod tests;

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DomEvent {
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

struct AppState {
    game: Game,
    input: MouseInput,
    last_frame: Duration,
    accumulator: Duration,
}

impl AppState {
    const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

    fn new(teams: Vec<Team>) -> Self {
        let (field, stick, skittles) = create_field();
        Self {
            game: Game::new(teams, field, stick, skittles),
            input: MouseInput::default(),
            last_frame: Duration::ZERO,
            accumulator: Duration::ZERO,
        }
    }

    fn phase(&self) -> Phase {
        if self.game.is_simulating() {
            Phase::Simulating
        } else if self.input.pointer_id().is_some() {
            Phase::Charging
        } else {
            Phase::Ready
        }
    }

    fn handle(&mut self, event: DomEvent) {
        match event {
            DomEvent::Press(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                if self.phase() == Phase::Ready
                    && !self.game.is_finished()
                    && self.input.press(
                        sample.pointer_id,
                        Some(MouseButton::from_web_code(sample.button)),
                        sample.target(),
                        now,
                    )
                {
                    let (target, speed) = self.input.preview(now).unwrap();
                    self.game.aim(target, speed);
                }
            }
            DomEvent::Move(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.input.move_to(sample.pointer_id, sample.target());
                if let Some((target, speed)) = self.input.preview(now) {
                    self.game.aim(target, speed);
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
                    self.game.aim(target, speed);
                    if self.game.throw() {
                        self.last_frame = now;
                        self.accumulator = Duration::ZERO;
                    }
                }
            }
            DomEvent::Cancel { pointer_id } => {
                self.input.cancel(Some(pointer_id));
                if self.input.pointer_id().is_none() {
                    self.game.cancel_aim();
                }
            }
            DomEvent::Blur => {
                self.input.cancel(None);
                self.game.cancel_aim();
            }
            DomEvent::Frame { time } => {
                if let Some(now) = timestamp(time) {
                    self.update(now);
                }
            }
        }
    }

    fn update(&mut self, now: Duration) {
        if let Some((target, speed)) = self.input.preview(now) {
            self.game.aim(target, speed);
        } else if self.game.is_simulating() {
            // タブ復帰時に大量の物理更新で描画を止めない。
            self.accumulator += now
                .saturating_sub(self.last_frame)
                .min(Self::MAX_FRAME_TIME);
            self.last_frame = now.max(self.last_frame);
            let step = Duration::from_secs_f32(TIME_STEP);
            while self.accumulator >= step && self.game.is_simulating() {
                self.game.update();
                self.accumulator -= step;
            }
        }
    }
}

fn render(state: &AppState) -> Element {
    let game = &state.game;
    let stick = game.stick();
    let finished = game.is_finished();
    let (phase, status) = if finished {
        ("finished", "試合終了")
    } else {
        match state.phase() {
            Phase::Ready => ("ready", "左ボタンを押して照準・蓄力を開始します。"),
            Phase::Charging => ("charging", "マウスで狙い、左ボタンを離すと投擲します。"),
            Phase::Simulating => ("simulating", "投擲中…"),
        }
    };
    let aiming = state.phase() == Phase::Charging;
    let skittles = game.skittles().iter().map(|skittle| {
        let id = skittle.as_ref().id();
        let class = if aiming && skittle.aimed() {
            "aimed"
        } else {
            ""
        };
        rsx! { g { key: "{id}", class, {super::skittle(skittle)} } }
    });
    let scene = rsx! {
        {skittles}
        if aiming { {aim(stick.position(), stick.target())} }
        {super::stick(stick)}
    };
    rsx! {
        style { {include_str!("style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {field(GuiField::BOUNDS, scene)}
                {speedometer(stick.speed(), MouseInput::MAX_SPEED)}
            }
            div { class: "scores",
                {scores(game.teams())}
                if finished { {result(game.winner())} }
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
