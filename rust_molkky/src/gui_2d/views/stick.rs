use crate::core::StickState;
use crate::gui_2d::{
    models::GuiStick,
    services::{Game, TIME_STEP},
};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::time::Duration;

#[cfg(test)]
#[path = "../../../tests/gui_2d/views/stick.rs"]
pub(super) mod tests;

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum DomEvent {
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

pub(super) fn phase(game: &Game) -> Phase {
    if game.is_simulating() {
        Phase::Simulating
    } else if game.stick().target().is_some() {
        Phase::Charging
    } else {
        Phase::Ready
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct PointerSample {
    pub pointer_id: i32,
    pub button: i16,
    pub cursor: [f64; 2],
    pub rect: [f64; 4],
    pub bounds: [f32; 4],
    pub time: f64,
}

impl PointerSample {
    fn target(&self) -> Option<[f32; 2]> {
        screen_to_world(self.cursor, self.rect, self.bounds)
    }
}

fn timestamp(milliseconds: f64) -> Option<Duration> {
    Duration::try_from_secs_f64(milliseconds / 1000.0).ok()
}

/// cursor/rectはCSSピクセル。rectはborderを除くSVG領域、boundsはworldの[x,y,幅,高さ]。
pub fn screen_to_world(cursor: [f64; 2], rect: [f64; 4], bounds: [f32; 4]) -> Option<[f32; 2]> {
    if !cursor.iter().chain(rect.iter()).all(|v| v.is_finite())
        || !bounds.iter().all(|v| v.is_finite())
        || rect[2] <= 0.0
        || rect[3] <= 0.0
        || bounds[2] <= 0.0
        || bounds[3] <= 0.0
    {
        return None;
    }
    let [x, y, width, height] = bounds.map(f64::from);
    // SVG既定のxMidYMid meetと同じ拡縮・中央寄せ。余白をフィールドに含めない。
    let scale = (rect[2] / width).min(rect[3] / height);
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let left = rect[0] + (rect[2] - width * scale) / 2.0;
    let top = rect[1] + (rect[3] - height * scale) / 2.0;
    let local_x = (cursor[0] - left) / scale;
    let local_y = (cursor[1] - top) / scale;
    if !(0.0..=width).contains(&local_x) || !(0.0..=height).contains(&local_y) {
        return None;
    }
    let point = [(x + local_x) as f32, (y + height - local_y) as f32];
    point.iter().all(|v| v.is_finite()).then_some(point)
}

struct Charge {
    pointer_id: i32,
    started_at: Duration,
    target: [f32; 2],
}

/// 棒の入力と、物理サービスを呼ぶフレーム時刻を管理する。
#[derive(Default)]
pub(super) struct StickInput {
    charge: Option<Charge>,
    last_frame: Duration,
    accumulator: Duration,
}

impl StickInput {
    const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

    pub(super) fn handle(&mut self, game: &mut Game, event: DomEvent) {
        match event {
            DomEvent::Press(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                if phase(game) == Phase::Ready
                    && !game.is_finished()
                    && self.press(
                        sample.pointer_id,
                        Some(MouseButton::from_web_code(sample.button)),
                        sample.target(),
                        now,
                    )
                {
                    let (target, speed) = self.preview(now).unwrap();
                    game.aim(target, speed);
                }
            }
            DomEvent::Move(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.move_to(sample.pointer_id, sample.target());
                if let Some((target, speed)) = self.preview(now) {
                    game.aim(target, speed);
                }
            }
            DomEvent::Release(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.move_to(sample.pointer_id, sample.target());
                if let Some((target, speed)) = self.release(
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    now,
                ) {
                    game.aim(target, speed);
                    if game.throw() {
                        self.last_frame = now;
                        self.accumulator = Duration::ZERO;
                    }
                }
            }
            DomEvent::Cancel { pointer_id } => {
                self.cancel(Some(pointer_id));
                if self.pointer_id().is_none() {
                    game.cancel_aim();
                }
            }
            DomEvent::Blur => {
                self.cancel(None);
                game.cancel_aim();
            }
            DomEvent::Frame { time } => {
                if let Some(now) = timestamp(time) {
                    self.update(game, now);
                }
            }
        }
    }

    fn update(&mut self, game: &mut Game, now: Duration) {
        if let Some((target, speed)) = self.preview(now) {
            game.aim(target, speed);
        } else if game.is_simulating() {
            // タブ復帰時に大量の物理更新で描画を止めない。
            self.accumulator += now
                .saturating_sub(self.last_frame)
                .min(Self::MAX_FRAME_TIME);
            self.last_frame = now.max(self.last_frame);
            let step = Duration::from_secs_f32(TIME_STEP);
            while self.accumulator >= step && game.is_simulating() {
                game.update();
                self.accumulator -= step;
            }
        }
    }

    pub const MAX_SPEED: f32 = 10.0;
    pub const CHARGE_PERIOD: Duration = Duration::from_secs(2);

    fn pointer_id(&self) -> Option<i32> {
        self.charge.as_ref().map(|charge| charge.pointer_id)
    }

    fn press(
        &mut self,
        pointer_id: i32,
        button: Option<MouseButton>,
        target: Option<[f32; 2]>,
        now: Duration,
    ) -> bool {
        if button != Some(MouseButton::Primary) || self.charge.is_some() {
            return false;
        }
        let Some(target) = target.filter(|p| p.iter().all(|v| v.is_finite())) else {
            return false;
        };
        self.charge = Some(Charge {
            pointer_id,
            started_at: now,
            target,
        });
        true
    }

    fn move_to(&mut self, pointer_id: i32, target: Option<[f32; 2]>) {
        let Some(charge) = self.charge.as_mut() else {
            return;
        };
        if charge.pointer_id == pointer_id
            && let Some(target) = target.filter(|p| p.iter().all(|v| v.is_finite()))
        {
            charge.target = target;
        }
    }

    fn preview(&self, now: Duration) -> Option<([f32; 2], f32)> {
        let charge = self.charge.as_ref()?;
        let elapsed = now.saturating_sub(charge.started_at).as_secs_f64();
        let phase =
            (elapsed % Self::CHARGE_PERIOD.as_secs_f64()) / Self::CHARGE_PERIOD.as_secs_f64();
        let speed = (1.0 - (phase * 2.0 - 1.0).abs()) as f32 * Self::MAX_SPEED;
        Some((charge.target, speed))
    }

    /// 座標を引数に取らないため、領域外での解放でも最後の有効な目標を使える。
    fn release(
        &mut self,
        pointer_id: i32,
        button: Option<MouseButton>,
        now: Duration,
    ) -> Option<([f32; 2], f32)> {
        if self.pointer_id() != Some(pointer_id) || button != Some(MouseButton::Primary) {
            return None;
        }
        let shot = self.preview(now);
        self.charge = None;
        shot
    }

    /// Someは該当pointerだけ、Noneはフォーカス喪失などによる全取消。
    fn cancel(&mut self, pointer_id: Option<i32>) {
        if pointer_id.is_none() || pointer_id == self.pointer_id() {
            self.charge = None;
        }
    }
}

/// フィールドのポインター入力と、共有Game内の棒の描画を担当する。
/// 角度はworld座標のラジアン、棒の長軸はローカルY軸。
#[component]
pub fn Stick(mut game: Signal<Game>) -> Element {
    let mut input = use_signal(StickInput::default);
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let model = *game.read().stick();
    let [x, y] = model.position();
    let id = model.as_ref().id();
    let state = match model.as_ref().state() {
        StickState::Held => "held",
        StickState::Move => "move",
        StickState::Stop => "stop",
    };
    let length = GuiStick::LENGTH;
    let width = 2.0 * GuiStick::RADIUS;
    let degrees = model.angle().to_degrees();
    let left = -width / 2.0;
    let top = -length / 2.0;
    let radius = width / 2.0;
    rsx! {
        g {
            id: "stick-{id}",
            "data-stick-state": state,
            transform: "translate({x} {y}) rotate({degrees})",
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("stick.js"));
                bridge.set(Some(eval));
                spawn(async move {
                    while let Ok(event) = eval.recv::<DomEvent>().await {
                        input.write().handle(&mut game.write(), event);
                    }
                    input.write().handle(&mut game.write(), DomEvent::Blur);
                    error.set(true);
                });
            },
            rect {
                x: "{left}", y: "{top}",
                width: "{width}", height: "{length}", rx: "{radius}",
                fill: "#995c32", stroke: "#493b28", stroke_width: "0.01"
            }
        }
        if model.as_ref().state() == StickState::Held {
            if let Some([tx, ty]) = model.target() {
                line { x1: "{x}", y1: "{y}", x2: "{tx}", y2: "{ty}",
                    stroke: "#355fba", stroke_width: "0.008", stroke_dasharray: "0.03 0.02" }
                circle { cx: "{tx}", cy: "{ty}", r: "0.025", fill: "none", stroke: "#355fba", stroke_width: "0.008" }
            }
        }
        if error() {
            text {
                role: "alert", transform: "scale(1 -1)",
                x: "-0.25", y: "-1.25", font_size: "0.025", fill: "#b00020",
                tspan { "入力の接続が停止しました。" }
                tspan { x: "-0.25", dy: "0.04", "画面を再読み込みしてください。" }
            }
        }
    }
}
