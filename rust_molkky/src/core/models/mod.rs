//! エンティティと値オブジェクトを含む、表示座標を持たないドメインモデル。
mod field;
mod game_status;
mod id;
mod name;
mod score;
mod skittle;
mod stick;
mod team;

pub use field::{Field, FieldGenerator};
pub use game_status::GameStatus;
pub use id::Id;
pub use name::{Name, NameError};
pub use score::{CONSECUTIVE_ZERO_LIMIT, Point, RESET_SCORE, ScoreBoard, TARGET_SCORE};
pub use skittle::Skittle;
pub(crate) use skittle::validate_unique_ids;
pub use stick::Stick;
pub use team::Team;
