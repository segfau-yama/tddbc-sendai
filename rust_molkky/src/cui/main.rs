use std::io::{self, BufRead, Write};

use super::{input, output};
use crate::{Molkky, ScoreState, SimpleMolkky, Team};

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

        if rules.judge_game(&teams).is_none() && teams.len() > 1 {
            let survivor = {
                let mut remaining = teams
                    .iter()
                    .enumerate()
                    .filter(|(_, team)| team.score_board().state() != ScoreState::Disqualified);
                match (remaining.next(), remaining.next()) {
                    (Some((index, _)), None) => Some(index),
                    _ => None,
                }
            };
            // 最後の生存者には投てきを記録せず、得点だけ50点を付与する。
            if let Some(index) = survivor {
                let board = teams[index].score_board().win_by_default();
                teams[index] = teams[index].with_score_board(board);
            }
        }

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
