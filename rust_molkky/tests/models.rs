// 削除途中のlibや編集中のサービスを復元せず、実際のモデルだけを検証する。
extern crate self as rust_molkky;

#[path = "../src/core/models/mod.rs"]
mod product_models;
pub use product_models::*;

#[path = "core/models/field.rs"]
mod field;
#[path = "core/models/id.rs"]
mod id;
#[path = "core/models/name.rs"]
mod name;
#[path = "core/models/score.rs"]
mod score;
#[path = "core/models/skittle.rs"]
mod skittle;
#[path = "core/models/stick.rs"]
mod stick;
#[path = "core/models/team.rs"]
mod team;
