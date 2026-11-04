use crate::Team;

pub struct GuiTeam(Team);

impl From<Team> for GuiTeam {
    fn from(value: Team) -> Self {
        Self(value)
    }
}

impl AsRef<Team> for GuiTeam {
    fn as_ref(&self) -> &Team {
        &self.0
    }
}
