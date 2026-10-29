use std::io::{self, Write};

use crate::{Point, ScoreState, Team};

fn prompt(writer: &mut impl Write, message: &str) -> io::Result<()> {
    write!(writer, "{message}")?;
    writer.flush()
}

pub fn team_count_prompt(writer: &mut impl Write) -> io::Result<()> {
    prompt(writer, "チーム数（1以上、qで終了）: ")
}

pub fn team_name_prompt(writer: &mut impl Write, number: usize) -> io::Result<()> {
    prompt(
        writer,
        &format!("チーム{number}の名前（1〜30文字、qで終了）: "),
    )
}

pub fn throw_prompt(writer: &mut impl Write, team: &Team) -> io::Result<()> {
    prompt(
        writer,
        &format!(
            "{}の投てき（番号を空白区切り、0はミス、qで終了）: ",
            team.name()
        ),
    )
}

pub fn error(writer: &mut impl Write, message: &str) -> io::Result<()> {
    writeln!(writer, "入力エラー: {message}")
}

pub fn scoreboard(writer: &mut impl Write, teams: &[Team]) -> io::Result<()> {
    writeln!(writer, "\n得点表")?;
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
            writer,
            "{marker} {}. {}: {}点 ({}投){state}",
            index + 1,
            team.name(),
            board.total(),
            board.history().len()
        )?;
    }
    Ok(())
}

pub fn throw_result(writer: &mut impl Write, team: &Team, point: Point) -> io::Result<()> {
    writeln!(
        writer,
        "{}の投てき: {}点 → {}点",
        team.name(),
        point.value(),
        team.score_board().total()
    )
}

pub fn finish(writer: &mut impl Write, winner: Option<&Team>) -> io::Result<()> {
    match winner {
        Some(team) => writeln!(
            writer,
            "{}の勝利です（{}点）",
            team.name(),
            team.score_board().total()
        ),
        None => writeln!(writer, "全チームが失格です。勝者はいません。"),
    }
}

pub fn quit(writer: &mut impl Write) -> io::Result<()> {
    writeln!(writer, "終了しました")
}
