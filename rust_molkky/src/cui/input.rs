use std::io::{self, BufRead, Write};

use super::{
    model::{TeamSetup, ThrowInput},
    output,
};
use crate::Team;

pub fn teams(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    names: &[String],
) -> io::Result<Option<Vec<Team>>> {
    if !names.is_empty() {
        return TeamSetup::from_names(names)
            .map(Some)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message));
    }
    let mut setup = loop {
        output::team_count_prompt(writer)?;
        let Some(line) = read_line(reader)? else {
            return Ok(None);
        };
        match TeamSetup::parse_count(&line) {
            Ok(setup) => break setup,
            Err(message) => output::error(writer, message)?,
        }
    };
    while !setup.is_complete() {
        output::team_name_prompt(writer, setup.next_number())?;
        let Some(line) = read_line(reader)? else {
            return Ok(None);
        };
        if let Err(message) = setup.add_name(&line) {
            output::error(writer, message)?;
        }
    }
    setup
        .finish()
        .map(Some)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))
}

pub fn throw_result(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    team: &Team,
) -> io::Result<Option<ThrowInput>> {
    loop {
        output::throw_prompt(writer, team)?;
        let Some(line) = read_line(reader)? else {
            return Ok(None);
        };
        match ThrowInput::parse(&line) {
            Ok(thrown) => return Ok(Some(thrown)),
            Err(message) => output::error(writer, message)?,
        }
    }
}

fn read_line(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
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
