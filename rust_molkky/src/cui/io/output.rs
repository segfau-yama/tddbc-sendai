use std::io::{self, Write};

use crate::{Point, ScoreState, Team};

pub struct Output<W> {
    writer: W,
}

impl<W: Write> Output<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    fn prompt(&mut self, message: &str) -> io::Result<()> {
        write!(self.writer, "{message}")?;
        self.writer.flush()
    }

    pub fn team_count_prompt(&mut self) -> io::Result<()> {
        self.prompt("チーム数（1以上、qで終了）: ")
    }

    pub fn team_name_prompt(&mut self, number: usize) -> io::Result<()> {
        self.prompt(&format!("チーム{number}の名前（1〜30文字、qで終了）: "))
    }

    pub fn throw_prompt(&mut self, team: &Team) -> io::Result<()> {
        self.prompt(&format!(
            "{}の投てき（狙う番号を空白区切り、0はミス、qで終了）: ",
            team.name()
        ))
    }

    pub fn error(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.writer, "入力エラー: {message}")
    }

    pub fn scoreboard(&mut self, teams: &[Team]) -> io::Result<()> {
        writeln!(self.writer, "\n得点表")?;
        for (index, team) in teams.iter().enumerate() {
            let marker = if team.turn() { "▶" } else { " " };
            let board = team.score_board();
            let state = match board.state() {
                ScoreState::Calculated => "",
                ScoreState::Reset => " [25点に補正]",
                ScoreState::Disqualified => " [失格]",
                ScoreState::Won => " [勝利]",
            };
            writeln!(
                self.writer,
                "{marker} {}. {}: {}点 ({}投){state}",
                index + 1,
                team.name(),
                board.total(),
                board.history().len()
            )?;
        }
        Ok(())
    }

    pub fn throw_result(&mut self, team: &Team, point: Point) -> io::Result<()> {
        writeln!(
            self.writer,
            "{}の投てき: {}点 → {}点",
            team.name(),
            point.value(),
            team.score_board().total()
        )
    }

    pub fn finish(&mut self, winner: Option<&Team>) -> io::Result<()> {
        match winner {
            Some(team) => writeln!(
                self.writer,
                "{}の勝利です（{}点）",
                team.name(),
                team.score_board().total()
            ),
            None => writeln!(self.writer, "全チームが失格です。勝者はいません。"),
        }
    }

    pub fn quit(&mut self) -> io::Result<()> {
        writeln!(self.writer, "終了しました")
    }
}
