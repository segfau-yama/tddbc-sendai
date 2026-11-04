use dioxus::html::input_data::MouseButton;
use std::time::Duration;

#[cfg(test)]
#[path = "../../../tests/gui_2d/controllers/mouse.rs"]
mod tests;

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
    pub fn target(&self) -> Option<[f32; 2]> {
        screen_to_world(self.cursor, self.rect, self.bounds)
    }
}

pub(crate) fn timestamp(milliseconds: f64) -> Option<Duration> {
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

/// マウスの照準・蓄力・解放・取消だけを管理する。
#[derive(Default)]
pub(super) struct MouseInput {
    charge: Option<Charge>,
}

impl MouseInput {
    pub const MAX_SPEED: f32 = 10.0;
    pub const CHARGE_PERIOD: Duration = Duration::from_secs(2);

    pub(crate) fn pointer_id(&self) -> Option<i32> {
        self.charge.as_ref().map(|charge| charge.pointer_id)
    }

    pub(crate) fn press(
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

    pub(crate) fn move_to(&mut self, pointer_id: i32, target: Option<[f32; 2]>) {
        let Some(charge) = self.charge.as_mut() else {
            return;
        };
        if charge.pointer_id == pointer_id
            && let Some(target) = target.filter(|p| p.iter().all(|v| v.is_finite()))
        {
            charge.target = target;
        }
    }

    pub(crate) fn preview(&self, now: Duration) -> Option<([f32; 2], f32)> {
        let charge = self.charge.as_ref()?;
        let elapsed = now.saturating_sub(charge.started_at).as_secs_f64();
        let phase =
            (elapsed % Self::CHARGE_PERIOD.as_secs_f64()) / Self::CHARGE_PERIOD.as_secs_f64();
        let speed = (1.0 - (phase * 2.0 - 1.0).abs()) as f32 * Self::MAX_SPEED;
        Some((charge.target, speed))
    }

    /// 座標を引数に取らないため、領域外での解放でも最後の有効な目標を使える。
    pub(crate) fn release(
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
    pub(crate) fn cancel(&mut self, pointer_id: Option<i32>) {
        if pointer_id.is_none() || pointer_id == self.pointer_id() {
            self.charge = None;
        }
    }
}
