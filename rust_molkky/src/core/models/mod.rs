//! エンティティと値オブジェクトを含む、表示座標を持たないドメインモデル。
mod field;
mod game_status;
mod score;
mod skittle;
mod stick;
mod team;

pub use field::Field;
pub use game_status::GameStatus;
pub use score::{CONSECUTIVE_ZERO_LIMIT, Point, RESET_SCORE, ScoreBoard, TARGET_SCORE};
pub use skittle::Skittle;
pub(crate) use skittle::validate_unique_numbers;
pub use stick::Stick;
pub use team::Team;
