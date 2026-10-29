//! エンティティと値オブジェクトを含む、表示座標を持たないドメインモデル。
mod field;
mod id;
mod name;
mod score;
mod skittle;
mod stick;
mod team;

pub use field::Field;
pub use id::Id;
pub use name::{Name, NameError};
pub use score::{ZERO_POINTS_LIMIT, Point, RESET_SCORE, ScoreBoard, ScoreState, TARGET_SCORE};
pub use skittle::{Skittle, SkittleState};
pub use stick::Stick;
pub use team::Team;
