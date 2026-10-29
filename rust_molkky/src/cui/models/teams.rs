use crate::{Name, Team};

#[derive(Default)]
pub struct Teams {
    pub teams: Vec<Team>,
}

impl Teams {
    pub fn from_names(names: &[String]) -> Result<Self, &'static str> {
        let mut teams = Self::default();
        for name in names {
            teams.add_name(name)?;
        }
        Ok(teams)
    }

    pub fn add_name(&mut self, input: &str) -> Result<(), &'static str> {
        if input.chars().any(char::is_control) {
            return Err("チーム名に制御文字は使用できません");
        }
        let name = input.trim();
        if name.is_empty() {
            return Err("チーム名は1文字以上必要です");
        }
        if name.chars().count() > 30 {
            return Err("チーム名は30文字以内にしてください");
        }
        self.teams.push(Team::new(Name::new(name)));
        Ok(())
    }
}
