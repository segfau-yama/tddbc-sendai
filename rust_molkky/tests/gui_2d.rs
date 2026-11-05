#[path = "gui_2d/models/field.rs"]
mod field;
#[path = "gui_2d/models/skittle.rs"]
mod skittle;
#[path = "gui_2d/models/stick.rs"]
mod stick;

#[cfg(feature = "gui_2d")]
#[path = "gui_2d/views/field.rs"]
mod field_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/views/result.rs"]
mod result_view;

#[cfg(feature = "gui_2d")]
#[path = "gui_2d/services/molkky.rs"]
mod molkky;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/views/score.rs"]
mod score_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/views/skittle.rs"]
mod skittle_view;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/views/speedometer.rs"]
mod speedometer_view;

#[cfg(feature = "gui_2d")]
#[path = "gui_2d/services/field.rs"]
mod field_service;
#[cfg(feature = "gui_2d")]
#[path = "gui_2d/services/teams.rs"]
mod teams_service;
