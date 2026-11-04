use crate::Team;

pub struct CuiTeam(Team);

impl From<Team> for CuiTeam {
    fn from(value: Team) -> Self {
        Self(value)
    }
}

impl AsRef<Team> for CuiTeam {
    fn as_ref(&self) -> &Team {
        &self.0
    }
}
