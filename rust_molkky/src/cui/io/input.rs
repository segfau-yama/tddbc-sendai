use std::io::{self, BufRead, Write};

use super::Output;
use crate::Team;
use crate::cui::models::Teams;

pub struct Input<R> {
    reader: R,
}

impl<R: BufRead> Input<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn teams(
        &mut self,
        output: &mut Output<impl Write>,
        names: &[String],
    ) -> io::Result<Option<Teams>> {
        if !names.is_empty() {
            return Teams::from_names(names)
                .map(Some)
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message));
        }
        let count = loop {
            output.team_count_prompt()?;
            let Some(line) = self.read_line()? else {
                return Ok(None);
            };
            match line.trim().parse::<usize>() {
                Ok(count) if count > 0 => break count,
                _ => output.error("チーム数は1以上の整数で入力してください")?,
            }
        };
        let mut teams = Teams::default();
        for number in 1..=count {
            loop {
                output.team_name_prompt(number)?;
                let Some(line) = self.read_line()? else {
                    return Ok(None);
                };
                match teams.add_name(&line) {
                    Ok(()) => break,
                    Err(message) => output.error(message)?,
                }
            }
        }
        Ok(Some(teams))
    }

    pub fn throw_result(
        &mut self,
        output: &mut Output<impl Write>,
        team: &Team,
    ) -> io::Result<Option<Vec<i32>>> {
        loop {
            output.throw_prompt(team)?;
            let Some(line) = self.read_line()? else {
                return Ok(None);
            };
            match parse_numbers(&line) {
                Ok(numbers) => return Ok(Some(numbers)),
                Err(message) => output.error(message)?,
            }
        }
    }

    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        if self.reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        if line.trim().eq_ignore_ascii_case("q") {
            return Ok(None);
        }
        Ok(Some(line))
    }
}

fn parse_numbers(input: &str) -> Result<Vec<i32>, &'static str> {
    if input.trim() == "0" {
        return Ok(Vec::new());
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
    Ok(numbers)
}
