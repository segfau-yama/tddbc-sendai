use super::team::TeamTui;
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Row, Table},
};

pub(crate) struct ScoreBoardTui {
    pub(crate) value: i32,
    pub(crate) target: i32,
    pub(crate) history: Vec<i32>,
    pub(crate) notice: String,
}
impl ScoreBoardTui {
    pub(crate) fn value_label(&self) -> String {
        format!("{} / {}", self.value, self.target)
    }
    pub(crate) fn recent_points_label(&self) -> String {
        self.history[self.history.len().saturating_sub(6)..]
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(crate) fn scores_table(teams: &[TeamTui]) -> Table<'_> {
    Table::new(
        teams.iter().map(TeamTui::row),
        [
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Min(16),
        ],
    )
    .header(
        Row::new(["", "チーム", "得点", "投数", "直近の点数"])
            .style(Style::default().fg(Color::Cyan)),
    )
    .block(Block::bordered().title("得点表"))
    .row_highlight_style(Style::default().bg(Color::DarkGray))
}
#[cfg(test)]
#[path = "../../tests/view/score.rs"]
mod tests;
