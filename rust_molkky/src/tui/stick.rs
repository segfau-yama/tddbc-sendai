use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Context,
};
use rust_molkky::core::Stick;

use super::{draw::CanvasDraw, position::PositionTui};

pub(super) struct StickTui<'a> {
    stick: &'a Stick,
}

impl<'a> StickTui<'a> {
    pub(super) fn new(stick: &'a Stick) -> Self {
        Self { stick }
    }

    pub(super) fn position(&self) -> (f64, f64) {
        let position = self.stick.position();
        PositionTui::new(&position).canvas()
    }

    pub(super) fn position_label(&self) -> String {
        PositionTui::new(&self.stick.position()).label()
    }
}

impl CanvasDraw for StickTui<'_> {
    fn draw(&self, context: &mut Context<'_>) {
        let (x, y) = self.position();
        context.print(
            x,
            y,
            Line::from(Span::styled("@", Style::default().fg(Color::Yellow))),
        );
    }
}
