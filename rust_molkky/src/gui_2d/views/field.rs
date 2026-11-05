use super::stick::StickInput;
use super::{Skittle, Stick, speedometer};
use crate::core::{Molkky, SkittleState, StickState};
use crate::gui_2d::{
    models::{GuiField, GuiSkittle, GuiStick},
    services::{GuiMolkky, TIME_STEP, physics},
};
use dioxus::prelude::*;
use std::{rc::Rc, time::Duration};

#[cfg(test)]
#[path = "../../../tests/gui_2d/views/field.rs"]
pub(super) mod tests;

/// 共有ワールドの更新と完了通知を一か所で行う。
#[component]
pub fn Field(
    field: GuiField,
    mut stick: Signal<GuiStick>,
    mut skittles: Signal<Vec<GuiSkittle>>,
    mut rules: Signal<GuiMolkky>,
    enabled: ReadSignal<bool>,
    on_complete: EventHandler<()>,
) -> Element {
    let mut clock = use_signal(|| Duration::ZERO);
    let mut surface = use_signal(|| None::<Rc<MountedData>>);
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let mounted = use_callback(move |event: MountedEvent| {
        surface.set(Some(event.data()));
        let mut eval = document::eval(include_str!("field.js"));
        bridge.set(Some(eval));
        spawn(async move {
            let mut last = Duration::ZERO;
            let mut accumulator = Duration::ZERO;
            while let Ok(time) = eval.recv::<f64>().await {
                let Ok(now) = Duration::try_from_secs_f64(time / 1000.0) else {
                    continue;
                };
                clock.set(now);
                let elapsed = now.saturating_sub(last);
                last = last.max(now);
                if enabled() {
                    let completed = advance(
                        &mut rules.write(),
                        &mut stick.write(),
                        &mut skittles.write(),
                        elapsed,
                        &mut accumulator,
                    );
                    if completed {
                        on_complete.call(());
                    }
                } else {
                    accumulator = Duration::ZERO;
                }
            }
            error.set(true);
        });
    });
    // 入力のモデル変化から照準だけを更新する。物理時間は進めない。
    use_effect(move || {
        let model = *stick.read();
        if model.as_ref().state() == StickState::Held {
            let targets = rules.peek().aim(&model, &skittles.peek());
            let updated: Vec<_> = skittles
                .peek()
                .iter()
                .map(|pin| {
                    let aimed = targets
                        .iter()
                        .any(|target| target.as_ref().id() == pin.as_ref().id());
                    pin.update(pin.position(), pin.angle(), pin.as_ref().state(), aimed)
                })
                .collect();
            if updated != *skittles.peek() {
                skittles.set(updated);
            }
        }
    });
    let model = *stick.read();
    let active = model.as_ref().state() == StickState::Move
        || skittles
            .read()
            .iter()
            .any(|s| s.as_ref().state() == SkittleState::Hit);
    let (phase, status) = if !enabled() {
        ("finished", "試合終了")
    } else if active {
        ("simulating", "投擲中…")
    } else if model.target().is_some() {
        ("charging", "マウスで狙い、左ボタンを離すと投擲します。")
    } else {
        ("ready", "左ボタンを押して照準・蓄力を開始します。")
    };
    let pins = skittles
        .read()
        .iter()
        .map(|pin| {
            let id = pin.as_ref().id();
            rsx! { Skittle { key: "{id}", model: *pin } }
        })
        .collect::<Vec<_>>();
    let scene = rsx! { Stick { model: stick, clock, surface, enabled, {pins.into_iter()} } };
    let field_id = field.as_ref().id();
    rsx! {
        div {
            id: "molkky-input", "data-phase": phase, "data-field": "{field_id}",
            p { role: "status", "{status}" }
            {field_shape(GuiField::BOUNDS, scene, mounted)}
            {speedometer(model.speed(), StickInput::MAX_SPEED)}
            if error() { p { role: "alert", "入力の接続が停止しました。画面を再読み込みしてください。" } }
        }
    }
}

pub(super) fn advance(
    rules: &mut GuiMolkky,
    stick: &mut GuiStick,
    skittles: &mut Vec<GuiSkittle>,
    elapsed: Duration,
    accumulator: &mut Duration,
) -> bool {
    if stick.as_ref().state() != StickState::Move
        && !skittles
            .iter()
            .any(|s| s.as_ref().state() == SkittleState::Hit)
    {
        *accumulator = Duration::ZERO;
        return stick.as_ref().state() == StickState::Stop;
    }
    // タブ復帰時も一度の描画で大量の物理時間を進めない。
    *accumulator += elapsed.min(Duration::from_millis(250));
    let step = Duration::from_secs_f32(TIME_STEP);
    while *accumulator >= step {
        (*skittles, *stick) = physics::step(rules, skittles, *stick);
        *accumulator -= step;
        if stick.as_ref().state() == StickState::Stop
            && skittles
                .iter()
                .all(|s| s.as_ref().state() != SkittleState::Hit)
        {
            *accumulator = Duration::ZERO;
            return true;
        }
    }
    false
}

/// boundsはworldの[x, y, 幅, 高さ]。childrenもworld座標で受け取る。
fn field_shape(
    bounds: [f32; 4],
    children: Element,
    mounted: EventHandler<MountedEvent>,
) -> Element {
    let [x, y, width, height] = bounds;
    let view_box = format!("{x} {} {width} {height}", -(y + height));
    let wall = GuiField::WALL_THICKNESS;
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            onmounted: move |event| mounted.call(event),
            view_box,
            role: "img",
            "aria-label": "モルックのフィールド",
            style: "display:block;width:100%;background:#e6efdc",
            g { transform: "scale(1 -1)",
                rect {
                    x: "{x + wall / 2.0}", y: "{y + wall / 2.0}",
                    width: "{width - wall}", height: "{height - wall}",
                    fill: "none", stroke: "#61734e", stroke_width: "{wall}"
                }
                {children}
            }
        }
    }
}
