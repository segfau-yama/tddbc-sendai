use std::io::{self, BufRead, Write};

use super::{input, output};
use crate::{Molkky, SimpleMolkky, Team};

pub fn run(reader: &mut impl BufRead, writer: &mut impl Write, names: &[String]) -> io::Result<()> {
    let Some(mut teams) = input::teams(reader, writer, names)? else {
        return output::quit(writer);
    };
    teams[0] = teams[0].with_turn(true);
    let rules = SimpleMolkky;
    loop {
        output::scoreboard(writer, &teams)?;
        let current = teams
            .iter()
            .position(Team::turn)
            .expect("手番を設定してから投てきする");
        let Some(thrown) = input::throw_result(reader, writer, &teams[current])? else {
            return output::quit(writer);
        };
        let point = rules.calc_score(thrown.skittles());
        let board = teams[current].score_board().add(point);
        teams[current] = teams[current].with_score_board(board);
        output::throw_result(writer, &teams[current], point)?;

        let next = rules.next_team(&teams).map(Team::id);
        for team in &mut teams {
            *team = team.with_turn(next == Some(team.id()));
        }
        if next.is_none() {
            output::scoreboard(writer, &teams)?;
            return output::finish(writer, rules.judge_game(&teams));
        }
    }
}
