use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Row, Table, TableState, Wrap},
};
use rust_molkky::core::{Molkky, RESET_SCORE, TARGET_SCORE};

use super::{field::FieldTui, stick::StickTui, team::TeamTui};

pub(super) const MIN_WIDTH: u16 = 72;
pub(super) const MIN_HEIGHT: u16 = 24;

pub(super) struct MolkkyTui<'a> {
    game: &'a Molkky,
    active: usize,
}

#[cfg(test)]
#[path = "../../tests/tui/molkky.rs"]
mod tests;

impl<'a> MolkkyTui<'a> {
    pub(super) fn new(game: &'a Molkky, active: usize) -> Self {
        Self { game, active }
    }

    pub(super) fn draw(&self, frame: &mut Frame, message: &str) {
        let area = frame.area();
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            frame.render_widget(Paragraph::new(format!(
                "端末を{MIN_WIDTH}列 × {MIN_HEIGHT}行以上に広げてください。\n現在: {}列 × {}行\nQ / Escで終了",
                area.width, area.height,
            )).block(Block::bordered().title("MÖLKKY")), area);
            return;
        }
        let [heading, scores, field, message_area, help] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .areas(area);
        let banner = match self.game.result() {
            Some(winner) => format!(
                " MÖLKKY  |  {}の勝利！  Rで新しいゲーム",
                self.game.teams()[winner].name()
            ),
            None if self.game.is_finished() => {
                " MÖLKKY  |  全チームが失格。勝者なし。Rで新しいゲーム".into()
            }
            None => format!(
                " MÖLKKY  |  {}の投てき  |  {}チーム",
                self.game.teams()[self.active].name(),
                self.game.teams().len()
            ),
        };
        frame.render_widget(
            Paragraph::new(banner).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            heading,
        );
        self.draw_scores(frame, scores);
        let [board, positions] =
            Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)])
                .areas(field);
        FieldTui::new(self.game.field(), self.game.stick()).draw(frame, board);
        FieldTui::new(self.game.field(), self.game.stick()).draw_positions(frame, positions);
        frame.render_widget(
            Paragraph::new(message).wrap(Wrap { trim: false }),
            message_area,
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
                    Span::raw(" 投てき  "),
                    Span::styled("矢印", Style::default().fg(Color::Yellow)),
                    Span::raw(" 位置移動  "),
                    Span::styled("R", Style::default().fg(Color::Yellow)),
                    Span::raw(" 最初から  "),
                    Span::styled("Q/Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" 終了"),
                ]),
                Line::from(format!(
                    "投てき位置: {}   {TARGET_SCORE}点ぴったりで勝利 / 超過すると{RESET_SCORE}点",
                    StickTui::new(self.game.stick()).position_label()
                )),
            ]),
            help,
        );
    }

    fn draw_scores(&self, frame: &mut Frame, area: Rect) {
        let winner = self.game.result();
        let rows = self.game.teams().iter().enumerate().map(|(index, team)| {
            TeamTui::new(team).row(index == self.active, winner == Some(index))
        });
        let table = Table::new(
            rows,
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
        .row_highlight_style(Style::default().bg(Color::DarkGray));
        frame.render_stateful_widget(
            table,
            area,
            &mut TableState::default().with_selected(Some(self.active)),
        );
    }
}
