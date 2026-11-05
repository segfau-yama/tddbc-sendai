use crate::core::{Move, StickState};
use crate::gui_2d::models::{GuiField, GuiStick};
use dioxus::{html::input_data::MouseButton, prelude::*};
use std::{rc::Rc, time::Duration};

/// rectはboundsと同じ縦横比のSVG領域(CSSピクセル)、boundsはworldの[x,y,幅,高さ]。
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
    let local_x = (cursor[0] - rect[0]) / rect[2];
    let local_y = (cursor[1] - rect[1]) / rect[3];
    if !(0.0..=1.0).contains(&local_x) || !(0.0..=1.0).contains(&local_y) {
        return None;
    }
    let point = [
        (x + local_x * width) as f32,
        (y + (1.0 - local_y) * height) as f32,
    ];
    point.iter().all(|v| v.is_finite()).then_some(point)
}

struct Charge {
    pointer_id: i32,
    started_at: Duration,
    target: [f32; 2],
}

#[derive(Default)]
pub(super) struct StickInput {
    charge: Option<Charge>,
}

impl StickInput {
    pub const MAX_SPEED: f32 = 10.0;
    pub const CHARGE_PERIOD: Duration = Duration::from_secs(2);

    fn pointer_id(&self) -> Option<i32> {
        self.charge.as_ref().map(|charge| charge.pointer_id)
    }

    fn press(
        &mut self,
        stick: &GuiStick,
        pointer_id: i32,
        button: Option<MouseButton>,
        target: Option<[f32; 2]>,
        now: Duration,
    ) -> bool {
        if stick.as_ref().state() != StickState::Held
            || button != Some(MouseButton::Primary)
            || self.charge.is_some()
        {
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

    fn preview(&self, stick: GuiStick, now: Duration) -> GuiStick {
        let Some(charge) = self.charge.as_ref() else {
            return stick;
        };
        if stick.as_ref().state() != StickState::Held {
            return stick;
        }
        let elapsed = now.saturating_sub(charge.started_at).as_secs_f64();
        let period = Self::CHARGE_PERIOD.as_secs_f64();
        let phase = (elapsed % period) / period;
        let speed = (1.0 - (phase * 2.0 - 1.0).abs()) as f32 * Self::MAX_SPEED;
        GuiStick::new(*stick.as_ref(), Some(charge.target), speed)
    }

    /// 座標を引数に取らないため、領域外での解放でも最後の有効な目標を使える。
    fn release(
        &mut self,
        stick: GuiStick,
        pointer_id: i32,
        button: Option<MouseButton>,
        now: Duration,
    ) -> GuiStick {
        if self.pointer_id() != Some(pointer_id) || button != Some(MouseButton::Primary) {
            return stick;
        }
        let moved = GuiStick::r#move(self.preview(stick, now));
        self.charge = None;
        if moved.as_ref().state() == StickState::Held {
            GuiStick::new(*stick.as_ref(), None, 0.0)
        } else {
            moved
        }
    }

    /// Someは該当pointerだけ、Noneはフォーカス喪失などによる全取消。
    fn cancel(&mut self, pointer_id: Option<i32>) -> bool {
        if self.charge.is_some() && (pointer_id.is_none() || pointer_id == self.pointer_id()) {
            self.charge = None;
            true
        } else {
            false
        }
    }
}

async fn target(event: &PointerData, surface: Option<Rc<MountedData>>) -> Option<[f32; 2]> {
    let rect = surface?.get_client_rect().await.ok()?;
    let p = event.client_coordinates();
    screen_to_world(
        [p.x, p.y],
        [
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        ],
        GuiField::BOUNDS,
    )
}

enum Input {
    Press(PointerEvent, Duration),
    Move(PointerEvent, Duration),
    Release(PointerEvent, Duration),
    Cancel(Option<i32>),
    Frame(Duration),
}

/// 棒のモデルと入力だけを扱う。childrenは同じ入力領域に配置する描画部品。
#[component]
pub fn Stick(
    mut model: Signal<GuiStick>,
    clock: ReadSignal<Duration>,
    surface: ReadSignal<Option<Rc<MountedData>>>,
    enabled: ReadSignal<bool>,
    #[props(default)] children: Element,
) -> Element {
    // 非同期の領域取得でも、押下・移動・解放の順序を保つ。
    let input = use_coroutine(move |mut rx: UnboundedReceiver<Input>| async move {
        let mut input = StickInput::default();
        while let Ok(event) = rx.recv().await {
            match event {
                Input::Press(event, now) => {
                    if !enabled() || event.pointer_type() != "mouse" {
                        continue;
                    }
                    let surface = surface.peek().clone();
                    let target = target(&event, surface).await;
                    if input.press(
                        &model.peek(),
                        event.pointer_id(),
                        event.trigger_button(),
                        target,
                        now,
                    ) {
                        let held = input.preview(*model.peek(), now);
                        model.set(held);
                    }
                }
                Input::Move(event, now) => {
                    if input.pointer_id() != Some(event.pointer_id()) {
                        continue;
                    }
                    let surface = surface.peek().clone();
                    input.move_to(event.pointer_id(), target(&event, surface).await);
                    let held = input.preview(*model.peek(), now);
                    model.set(held);
                }
                Input::Release(event, now) => {
                    let surface = surface.peek().clone();
                    input.move_to(event.pointer_id(), target(&event, surface).await);
                    let moved = input.release(
                        *model.peek(),
                        event.pointer_id(),
                        event.trigger_button(),
                        now,
                    );
                    model.set(moved);
                }
                Input::Cancel(id) => {
                    if input.cancel(id) && model.peek().as_ref().state() == StickState::Held {
                        let held = GuiStick::new(*model.peek().as_ref(), None, 0.0);
                        model.set(held);
                    }
                }
                Input::Frame(now) => {
                    let next = input.preview(*model.peek(), now);
                    if next != *model.peek() {
                        model.set(next);
                    }
                }
            }
        }
    });
    use_effect(move || {
        let now = clock();
        if enabled() {
            input.send(Input::Frame(now));
        } else {
            input.send(Input::Cancel(None));
        }
    });
    let stick = *model.read();
    let [x, y] = stick.position();
    let id = stick.as_ref().id();
    let state = match stick.as_ref().state() {
        StickState::Held => "held",
        StickState::Move => "move",
        StickState::Stop => "stop",
    };
    let [bx, by, bw, bh] = GuiField::BOUNDS;
    let width = 2.0 * GuiStick::RADIUS;
    let length = GuiStick::LENGTH;
    let degrees = stick.angle().to_degrees();
    rsx! {
        g {
            "data-stick-input": "true",
            onpointerdown: move |event| input.send(Input::Press(event, clock())),
            onpointermove: move |event| {
                if !event.held_buttons().contains(MouseButton::Primary) {
                    if event.trigger_button() == Some(MouseButton::Primary) { input.send(Input::Release(event, clock())); }
                    else { input.send(Input::Cancel(Some(event.pointer_id()))); }
                } else { input.send(Input::Move(event, clock())); }
            },
            onpointerup: move |event| input.send(Input::Release(event, clock())),
            onpointercancel: move |event| input.send(Input::Cancel(Some(event.pointer_id()))),
            onlostpointercapture: move |event| input.send(Input::Cancel(Some(event.pointer_id()))),
            rect { x: "{bx}", y: "{by}", width: "{bw}", height: "{bh}", fill: "transparent" }
            {children}
            g {
                id: "stick-{id}", "data-stick-state": state,
                transform: "translate({x} {y}) rotate({degrees})",
                rect {
                    x: "{-width / 2.0}", y: "{-length / 2.0}",
                    width: "{width}", height: "{length}", rx: "{width / 2.0}",
                    fill: "#995c32", stroke: "#493b28", stroke_width: "0.01"
                }
            }
            if stick.as_ref().state() == StickState::Held {
                if let Some([tx, ty]) = stick.target() {
                    line { x1: "{x}", y1: "{y}", x2: "{tx}", y2: "{ty}", stroke: "#355fba", stroke_width: "0.008", stroke_dasharray: "0.03 0.02" }
                    circle { cx: "{tx}", cy: "{ty}", r: "0.025", fill: "none", stroke: "#355fba", stroke_width: "0.008" }
                }
            }
        }
    }
}
