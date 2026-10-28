mod placement;
mod presentation;
mod progress;

pub(crate) use placement::{FieldLayout, relocate_skittles};
pub(crate) use presentation::{compose_game, throw_message};

pub(crate) use progress::winning_team;
