use super::{
    models::{Game, GamePhase, GuiField, GuiStick},
    services::GuiMolkky,
};
use crate::core::{Molkky, Skittle, Team};
use dioxus::html::input_data::MouseButton;
use std::time::Duration;

#[cfg(test)]
#[path = "../../tests/gui_2d/controller.rs"]
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

/// 入力とゲーム進行を管理する。DOM接続と描画はappが担当する。
pub struct Controller {
    game: Game,
    rules: GuiMolkky,
    charge: Option<Charge>,
    last_frame: Duration,
    accumulator: Duration,
}

impl Controller {
    pub fn new(teams: Vec<Team>) -> Self {
        let mut game = Game::new(teams);
        for team in &mut game.teams {
            *team = team.with_turn(false);
        }
        let placements = game.field.initial_positions(GuiMolkky::SKITTLE_RADIUS);
        let rules = GuiMolkky::new(&game.stick, GuiField::THROW_POSITION, &placements);
        let mut controller = Self {
            game,
            rules,
            charge: None,
            last_frame: Duration::ZERO,
            accumulator: Duration::ZERO,
        };
        controller.update_game_state();
        controller
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub(crate) fn rules(&self) -> &GuiMolkky {
        &self.rules
    }

    pub const MAX_SPEED: f32 = 10.0;
    pub const CHARGE_PERIOD: Duration = Duration::from_secs(2);
    const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

    pub(crate) fn handle(&mut self, event: DomEvent) {
        match event {
            DomEvent::Press(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                if self.press(
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    sample.target(),
                    now,
                ) {
                    self.game.stick = self.preview(now).unwrap();
                    self.game.phase = GamePhase::Charging;
                }
            }
            DomEvent::Move(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.move_to(sample.pointer_id, sample.target());
                if let Some(stick) = self.preview(now) {
                    self.game.stick = stick;
                }
            }
            DomEvent::Release(sample) => {
                let Some(now) = timestamp(sample.time) else {
                    return;
                };
                self.move_to(sample.pointer_id, sample.target());
                if let Some(stick) = self.release(
                    sample.pointer_id,
                    Some(MouseButton::from_web_code(sample.button)),
                    now,
                ) {
                    self.game.stick = stick;
                    if self.rules.throw(&stick) {
                        self.game.phase = GamePhase::Simulating;
                        self.last_frame = now;
                        self.accumulator = Duration::ZERO;
                    } else {
                        self.reset_input();
                        self.game.phase = GamePhase::Ready;
                    }
                }
            }
            DomEvent::Cancel { pointer_id } => self.cancel_pointer(pointer_id),
            DomEvent::Blur => self.cancel(),
            DomEvent::Frame { time } => {
                if let Some(now) = timestamp(time) {
                    self.update(now);
                }
            }
        }
    }

    fn reset_input(&mut self) {
        self.charge = None;
        self.game.stick = GuiStick::new(*self.game.stick.stick(), None, 0.0);
    }

    fn update(&mut self, now: Duration) {
        if self.game.phase == GamePhase::Charging {
            if let Some(stick) = self.preview(now) {
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
        self.reset_input();
        self.update_game_state();
    }

    fn update_game_state(&mut self) {
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
        if self.game.phase != GamePhase::Ready
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

    fn preview(&self, now: Duration) -> Option<GuiStick> {
        let charge = self.charge.as_ref()?;
        let elapsed = now.saturating_sub(charge.started_at).as_secs_f64();
        let phase =
            (elapsed % Self::CHARGE_PERIOD.as_secs_f64()) / Self::CHARGE_PERIOD.as_secs_f64();
        let speed = (1.0 - (phase * 2.0 - 1.0).abs()) as f32 * Self::MAX_SPEED;
        Some(GuiStick::new(
            *self.game.stick.stick(),
            Some(charge.target),
            speed,
        ))
    }

    /// 座標を引数に取らないため、領域外での解放でも最後の有効な目標を使える。
    fn release(
        &mut self,
        pointer_id: i32,
        button: Option<MouseButton>,
        now: Duration,
    ) -> Option<GuiStick> {
        if self.pointer_id() != Some(pointer_id) || button != Some(MouseButton::Primary) {
            return None;
        }
        let shot = self.preview(now);
        self.charge = None;
        shot
    }

    /// pointercancel/lostpointercapture用。他のpointerの取消は無視する。
    fn cancel_pointer(&mut self, pointer_id: i32) {
        if self.pointer_id() == Some(pointer_id) {
            self.cancel();
        }
    }

    /// focus喪失など、入力全体を破棄する場合に使う。
    fn cancel(&mut self) {
        if self.game.phase == GamePhase::Charging {
            self.reset_input();
            self.game.phase = GamePhase::Ready;
        } else {
            self.charge = None;
        }
    }
}
