use super::{GuiField, GuiStick};
use crate::core::Team;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Ready,
    Charging,
    Simulating,
    Finished,
}

/// Controllerが更新するGUIの状態。ルールと状態遷移の処理は持たない。
#[derive(Clone)]
pub struct Game {
    pub teams: Vec<Team>,
    pub field: GuiField,
    pub stick: GuiStick,
    pub phase: GamePhase,
}

impl Game {
    pub fn new(teams: Vec<Team>) -> Self {
        Self {
            teams,
            field: GuiField::default(),
            stick: GuiStick::default(),
            phase: GamePhase::Ready,
        }
    }
}
