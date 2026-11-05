use super::stick::StickInput;
use super::{Skittle, Speedometer, Stick};
use crate::core::{Molkky, SkittleState, StickState};
use crate::gui_2d::{
    models::{GuiField, GuiSkittle, GuiStick},
    services::{GuiMolkky, TIME_STEP},
};
use dioxus::prelude::*;
use rapier2d::prelude::{Rotation, Vector};
use std::{rc::Rc, time::Duration};

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
    let field_id = field.as_ref().id();
    rsx! {
        div {
            id: "molkky-input",
            "data-phase": phase,
            "data-field": "{field_id}",
            p { role: "status", "{status}" }
            FieldSurface {
                onmounted: mounted,
                Stick {
                    model: stick,
                    clock,
                    surface,
                    enabled,
                    for pin in skittles.read().iter() {
                        Skittle {
                            key: "{pin.as_ref().id()}",
                            model: *pin,
                        }
                    }
                }
            }
            Speedometer {
                speed: model.speed(),
                max_speed: StickInput::MAX_SPEED,
            }
            if error() {
                p {
                    role: "alert",
                    "入力の接続が停止しました。画面を再読み込みしてください。"
                }
            }
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
        (*skittles, *stick) = {
            let body = &mut rules.world.bodies[rules.stick_body];
            // moveが用意した初速を一度反映し、以降は物理計算の結果を使う。
            if stick.as_ref().state() == StickState::Move
                && (body.translation().to_array() != stick.position()
                    || body.linvel().to_array() != stick.velocity()
                    || body.rotation().angle() != stick.angle()
                    || body.angvel() != stick.angular_velocity())
            {
                body.set_translation(Vector::from_array(stick.position()), false);
                body.set_rotation(Rotation::from_angle(stick.angle()), false);
                body.set_linvel(Vector::from_array(stick.velocity()), false);
                body.set_angvel(stick.angular_velocity(), false);
                body.wake_up(true);
            }
            // 棒と全スキットルが同じ時刻の衝突結果を使う。
            rules.world.step();
            for pair in rules.world.narrow_phase.contact_pairs() {
                let handles = [pair.collider1, pair.collider2];
                if handles.iter().all(|handle| {
                    rules.world.colliders[*handle]
                        .parent()
                        .is_none_or(|body| rules.world.bodies[body].is_sleeping())
                }) {
                    continue;
                }
                let impulse = pair.total_impulse_magnitude();
                for key in [(handles[0], handles[1]), (handles[1], handles[0])] {
                    let peak = rules.impacts.entry(key).or_default();
                    *peak = peak.max(impulse);
                }
            }
            let body = &rules.world.bodies[rules.stick_body];
            let updated = stick.update(
                body.translation().to_array(),
                body.rotation().angle(),
                if body.is_sleeping() {
                    [0.0; 2]
                } else {
                    body.linvel().to_array()
                },
                if body.is_sleeping() {
                    0.0
                } else {
                    body.angvel()
                },
            );
            let mut skittles: Vec<_> = skittles
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
            let contacts: Vec<_> = rules
                .impacts
                .iter()
                .filter_map(|(&(a, b), &impulse)| {
                    if impulse <= 0.0 || a.into_raw_parts() >= b.into_raw_parts() {
                        return None;
                    }
                    let id = |handle| {
                        if handle == rules.stick_collider {
                            Some(rules.stick_id)
                        } else {
                            rules.skittle_ids.get(&handle).copied()
                        }
                    };
                    Some((id(a)?, id(b)?))
                })
                .collect();
            // 接触履歴のないHitで再開した場合も、停止時に既存のhitで確定する。
            for skittle in skittles.iter_mut().filter(|s| {
                s.as_ref().state() == SkittleState::Hit
                    && !contacts
                        .iter()
                        .any(|&(a, b)| a == s.as_ref().id() || b == s.as_ref().id())
            }) {
                *skittle = rules.hit(&updated, skittle);
            }
            for (a, b) in contacts {
                let stick_id = updated.as_ref().id();
                if a == stick_id || b == stick_id {
                    let id = if a == stick_id { b } else { a };
                    if let Some(skittle) = skittles.iter_mut().find(|s| s.as_ref().id() == id) {
                        *skittle = rules.hit(&updated, skittle);
                    }
                } else if let (Some(left), Some(right)) = (
                    skittles.iter().position(|s| s.as_ref().id() == a),
                    skittles.iter().position(|s| s.as_ref().id() == b),
                ) {
                    let first = skittles[left];
                    let second = skittles[right];
                    skittles[left] = rules.chain_hit(&second, &first);
                    skittles[right] = rules.chain_hit(&first, &second);
                }
            }
            (skittles, updated)
        };
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

#[component]
fn FieldSurface(onmounted: EventHandler<MountedEvent>, children: Element) -> Element {
    let [x, y, width, height] = GuiField::BOUNDS;
    let view_box = format!("{x} {} {width} {height}", -(y + height));
    let wall = GuiField::WALL_THICKNESS;
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            onmounted: move |event| onmounted.call(event),
            view_box,
            role: "img",
            "aria-label": "モルックのフィールド",
            style: "display:block;background:#e6efdc",
            g { transform: "scale(1 -1)",
                rect {
                    x: x + wall / 2.0,
                    y: y + wall / 2.0,
                    width: width - wall,
                    height: height - wall,
                    fill: "none",
                    stroke: "#61734e",
                    stroke_width: wall,
                }
                {children}
            }
        }
    }
}
