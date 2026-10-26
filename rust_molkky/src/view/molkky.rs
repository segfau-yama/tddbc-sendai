use super::{field::FieldTui, score::scores_table, team::TeamTui};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, TableState, Widget, Wrap},
};

pub(crate) const MIN_WIDTH: u16 = 72;
pub(crate) const MIN_HEIGHT: u16 = 24;

pub(crate) struct MolkkyTui {
    pub(crate) teams: Vec<TeamTui>,
    pub(crate) field: FieldTui,
    pub(crate) active: usize,
    pub(crate) banner: String,
    pub(crate) message: String,
    pub(crate) help: String,
}
#[cfg(test)]
#[path = "../../tests/view/molkky.rs"]
mod tests;

impl Widget for MolkkyTui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            Paragraph::new(format!(
                "端末を{MIN_WIDTH}列 × {MIN_HEIGHT}行以上に広げてください。\n現在: {}列 × {}行\nQ / Escで終了",
                area.width, area.height,
            )).block(Block::bordered().title("MÖLKKY")).render(area, buf);
            return;
        }
        let [heading, scores, field, message_area, help] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length((self.teams.len() + 3).min(8) as u16),
            Constraint::Min(8),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .areas(area);
        Paragraph::new(self.banner.as_str())
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .render(heading, buf);
        ratatui::widgets::StatefulWidget::render(
            scores_table(&self.teams),
            scores,
            buf,
            &mut TableState::default().with_selected(Some(
                self.teams
                    .iter()
                    .position(|team| team.winner)
                    .unwrap_or(self.active),
            )),
        );
        let [board, positions] =
            Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)])
                .areas(field);
        self.field.positions_table().render(positions, buf);
        self.field.render(board, buf);
        Paragraph::new(self.message)
            .wrap(Wrap { trim: false })
            .render(message_area, buf);
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" 投てき  "),
                Span::styled("矢印", Style::default().fg(Color::Yellow)),
                Span::raw(" 表示位置移動  "),
                Span::styled("R", Style::default().fg(Color::Yellow)),
                Span::raw(" 最初から  "),
                Span::styled("Q/Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" 終了"),
            ]),
            Line::from(self.help),
        ])
        .render(help, buf);
    }
}
