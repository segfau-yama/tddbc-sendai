use ratatui::{
    style::{Color, Style},
    widgets::{Cell, Row},
};
use rust_molkky::core::{Point, Team};

use super::{point::PointTui, score::ScoreBoardTui};

pub(super) struct TeamTui<'a> {
    team: &'a Team,
}

#[cfg(test)]
#[path = "../../tests/tui/team.rs"]
mod tests;

impl<'a> TeamTui<'a> {
    pub(super) fn new(team: &'a Team) -> Self {
        Self { team }
    }

    pub(super) fn row(&self, active: bool, winner: bool) -> Row<'a> {
        let board = ScoreBoardTui::new(self.team.score_board());
        let marker = if winner {
            "勝"
        } else if self.team.score_board().is_disqualified() {
            "失格"
        } else if active {
            "▶"
        } else {
            ""
        };
        Row::new([
            Cell::from(marker),
            Cell::from(self.team.name()),
            Cell::from(board.value_label()),
            Cell::from(board.throw_count_label()),
            Cell::from(board.recent_points_label()),
        ])
        .style(if winner {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
    }

    pub(super) fn throw_message(&self, turn: usize, before: i32, point: &Point) -> String {
        let board = ScoreBoardTui::new(self.team.score_board());
        format!(
            "第{turn}投  {}: {}点  得点 {before} → {}{}",
            self.team.name(),
            PointTui::new(point).label(),
            self.team.score_board().value(),
            board.change_notice()
        )
    }
}
