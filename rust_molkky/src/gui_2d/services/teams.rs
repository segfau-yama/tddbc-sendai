use crate::{Name, Team};

pub fn from_names(names: &[&str]) -> Vec<Team> {
    names
        .iter()
        .map(|name| Team::new(Name::new(name)))
        .collect()
}
