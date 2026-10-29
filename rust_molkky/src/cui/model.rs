use crate::{Name, Skittle, Team};

const COUNT_ERROR: &str = "チーム数は1以上の整数で入力してください";

/// 入力を検証し、指定した人数のチームを入力順に組み立てる。
pub struct TeamSetup {
    count: usize,
    teams: Vec<Team>,
}

impl TeamSetup {
    pub fn new(count: usize) -> Result<Self, &'static str> {
        if count == 0 {
            return Err(COUNT_ERROR);
        }
        Ok(Self {
            count,
            teams: Vec::new(),
        })
    }

    pub fn parse_count(input: &str) -> Result<Self, &'static str> {
        Self::new(input.trim().parse().map_err(|_| COUNT_ERROR)?)
    }

    pub fn from_names(names: &[String]) -> Result<Vec<Team>, &'static str> {
        let mut setup = Self::new(names.len())?;
        for name in names {
            setup.add_name(name)?;
        }
        setup.finish()
    }

    pub fn add_name(&mut self, input: &str) -> Result<(), &'static str> {
        if self.is_complete() {
            return Err("指定したチーム数を超えています");
        }
        // trimで消える改行やタブも、入力の制御文字として拒否する。
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

    pub fn next_number(&self) -> usize {
        self.teams.len() + 1
    }

    pub fn is_complete(&self) -> bool {
        self.teams.len() == self.count
    }

    pub fn finish(self) -> Result<Vec<Team>, &'static str> {
        if !self.is_complete() {
            return Err("指定したチーム数の名前が必要です");
        }
        Ok(self.teams)
    }
}

/// 番号の範囲と重複を検証してから、採点対象の個体を生成する。
pub struct ThrowInput {
    skittles: Vec<Skittle>,
}

impl ThrowInput {
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        if input.trim() == "0" {
            return Ok(Self {
                skittles: Vec::new(),
            });
        }
        let mut numbers = Vec::new();
        for token in input.split_whitespace() {
            let number: i32 = token.parse().map_err(|_| "番号は整数で入力してください")?;
            if !(1..=12).contains(&number) {
                return Err("番号は1〜12、ミスは0だけを入力してください");
            }
            if numbers.contains(&number) {
                return Err("同じ番号は1回だけ入力してください");
            }
            numbers.push(number);
        }
        if numbers.is_empty() {
            return Err("倒れた番号、またはミスの0を入力してください");
        }
        Ok(Self {
            skittles: numbers.into_iter().map(Skittle::new).collect(),
        })
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }
}
