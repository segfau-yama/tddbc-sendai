#![doc = include_str!("../tests/api.md")]

mod field;
mod molkky;
mod movement;
mod score;
mod skittle;
mod stick;
mod team;
mod throw;
pub use field::Field;
pub use molkky::Molkky;
pub use movement::relocate_skittles;
pub use score::{CONSECUTIVE_ZERO_LIMIT, Point, RESET_SCORE, ScoreBoard, TARGET_SCORE, calc_score};
pub use skittle::{Position, Skittle};
pub use stick::Stick;
pub use team::Team;
pub use throw::{fall_probability, throw_stick};
