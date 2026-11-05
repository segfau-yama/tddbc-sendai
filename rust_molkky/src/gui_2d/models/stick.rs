use super::GuiField;
use crate::core::{Move, Stick, StickState, Throw};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiStick {
    stick: Stick,
    target: Option<[f32; 2]>,
    speed: f32,
    position: [f32; 2],
    angle: f32,
    velocity: [f32; 2],
    angular_velocity: f32,
}

impl Default for GuiStick {
    fn default() -> Self {
        Self::new(Stick::new(), None, 0.0)
    }
}

impl GuiStick {
    // 寸法はm、質量はkg。
    pub const RADIUS: f32 = 0.0275;
    pub const LENGTH: f32 = 0.225;
    pub const MASS: f32 = 0.25;

    /// targetと位置はm、speedはm/s。新しい保持状態を同じIdで用意する。
    pub fn new(stick: Stick, target: Option<[f32; 2]>, speed: f32) -> Self {
        Self {
            stick: stick.with_state(StickState::Held),
            target,
            speed,
            position: GuiField::THROW_POSITION,
            angle: 0.0,
            velocity: [0.0; 2],
            angular_velocity: 0.0,
        }
    }

    pub fn stick(&self) -> &Stick {
        &self.stick
    }
    pub fn target(&self) -> Option<[f32; 2]> {
        self.target
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn position(&self) -> [f32; 2] {
        self.position
    }
    pub fn angle(&self) -> f32 {
        self.angle
    }
    pub fn velocity(&self) -> [f32; 2] {
        self.velocity
    }
    pub fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    pub fn update(
        self,
        position: [f32; 2],
        angle: f32,
        velocity: [f32; 2],
        angular_velocity: f32,
    ) -> Self {
        Self::r#move(Self {
            position,
            angle,
            velocity,
            angular_velocity,
            ..self
        })
    }
}

impl AsRef<Stick> for GuiStick {
    fn as_ref(&self) -> &Stick {
        &self.stick
    }
}

impl Throw for GuiStick {
    fn throw(stick: Self) -> Self {
        let Some(target) = stick.target else {
            return stick;
        };
        let [x, y] = [target[0] - stick.position[0], target[1] - stick.position[1]];
        let length = x.hypot(y);
        if stick.stick.state() != StickState::Held
            || !length.is_finite()
            || length == 0.0
            || !stick.speed.is_finite()
            || stick.speed <= 0.0
        {
            return stick;
        }
        Self {
            stick: stick.stick.with_state(StickState::Move),
            velocity: [x / length * stick.speed, y / length * stick.speed],
            angle: y.atan2(x) - std::f32::consts::FRAC_PI_2,
            ..stick
        }
    }
}

impl Move for GuiStick {
    fn r#move(stick: Self) -> Self {
        if stick.stick.state() == StickState::Held {
            return stick;
        }
        let state = if stick.velocity == [0.0; 2] && stick.angular_velocity == 0.0 {
            StickState::Stop
        } else {
            StickState::Move
        };
        Self {
            stick: stick.stick.with_state(state),
            speed: stick.velocity[0].hypot(stick.velocity[1]),
            ..stick
        }
    }
}
