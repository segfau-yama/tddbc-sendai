#[path = "gui_2d/models/field.rs"]
mod field;
#[path = "gui_2d/models/skittle.rs"]
mod skittle;
#[path = "gui_2d/models/stick.rs"]
mod stick;

#[cfg(feature = "gui_2d")]
#[path = "gui_2d/services/field.rs"]
mod field_service;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/services/teams.rs"]
mod teams_service;
