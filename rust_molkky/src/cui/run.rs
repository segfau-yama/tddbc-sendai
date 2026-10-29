use std::io::{self, BufRead, Write};

use super::{
    io::{Input, Output},
    models::{CuiStick, Field},
    services::CuiMolkky,
};
use crate::{Molkky, Team};

pub fn run(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    names: &[String],
    stick: &CuiStick,
) -> io::Result<()> {
    let mut input = Input::new(reader);
    let mut output = Output::new(writer);
    let Some(mut participants) = input.teams(&mut output, names)? else {
        return output.quit();
    };
    let teams = &mut participants.teams;
    teams[0] = teams[0].with_turn(true);
    let field = Field::default();
    loop {
        output.scoreboard(teams)?;
        let current = teams
            .iter()
            .position(Team::turn)
            .expect("手番を設定してから投てきする");
        let Some(numbers) = input.throw_result(&mut output, &teams[current])? else {
            return output.quit();
        };
        let rules = CuiMolkky::new(numbers);
        let targets = rules.aim(stick, &field.skittles);
        let fallen = rules.hit(stick, &targets);
        let point = rules.calc_score(&fallen);
        let board = teams[current].score_board().add(point);
        teams[current] = teams[current].with_score_board(board);
        output.throw_result(&teams[current], point)?;

        let next = rules.next_team(teams).map(Team::id);
        let winner = rules.judge_game(teams).map(Team::id);
        for team in teams.iter_mut() {
            *team = team.with_turn(next == Some(team.id()));
            if winner == Some(team.id()) {
                *team = team.with_score_board(team.score_board().with_win());
            }
        }
        if next.is_none() {
            output.scoreboard(teams)?;
            return output.finish(rules.judge_game(teams));
        }
    }
}
