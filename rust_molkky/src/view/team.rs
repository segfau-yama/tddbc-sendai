use super::score::ScoreBoardTui;
use ratatui::{
    style::{Color, Style},
    widgets::{Cell, Row},
};

pub(crate) struct TeamTui {
    pub(crate) name: String,
    pub(crate) board: ScoreBoardTui,
    pub(crate) active: bool,
    pub(crate) winner: bool,
    pub(crate) disqualified: bool,
}
impl TeamTui {
    pub(crate) fn row(&self) -> Row<'_> {
        let marker = if self.winner {
            "勝"
        } else if self.disqualified {
            "失格"
        } else if self.active {
            "▶"
        } else {
            ""
        };
        Row::new([
            Cell::from(marker),
            Cell::from(self.name.as_str()),
            Cell::from(self.board.value_label()),
            Cell::from(self.board.throw_count_label()),
            Cell::from(self.board.recent_points_label()),
        ])
        .style(if self.winner {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
    }
}
#[cfg(test)]
#[path = "../../tests/view/team.rs"]
mod tests;
