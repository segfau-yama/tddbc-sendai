#[path = "gui_2d/models/field.rs"]
mod field;
#[path = "gui_2d/models/game.rs"]
mod game;
#[path = "gui_2d/models/stick.rs"]
mod stick;

#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/field.rs"]
mod field_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/result.rs"]
mod result_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/score.rs"]
mod score_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/skittle.rs"]
mod skittle_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/speedometer.rs"]
mod speedometer_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/view/stick.rs"]
mod stick_view;
