use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Context,
};
use rust_molkky::core::Stick;

pub(super) struct StickTui<'a> {
    stick: &'a Stick,
}

impl<'a> StickTui<'a> {
    pub(super) fn new(stick: &'a Stick) -> Self {
        Self { stick }
    }

    pub(super) fn position(&self) -> (f64, f64) {
        let position = self.stick.position();
        (f64::from(position.x()), f64::from(position.y()))
    }

    pub(super) fn draw(&self, context: &mut Context<'_>) {
        let (x, y) = self.position();
        context.print(
            x,
            y,
            Line::from(Span::styled("@", Style::default().fg(Color::Yellow))),
        );
    }
}
