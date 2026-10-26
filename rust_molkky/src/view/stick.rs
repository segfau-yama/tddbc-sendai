use super::{draw::CanvasDraw, position::Position};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Context,
};

pub(crate) struct StickTui {
    pub(crate) position: Position,
}
impl StickTui {
    pub(crate) fn position(&self) -> (f64, f64) {
        self.position.canvas()
    }
}
impl CanvasDraw for StickTui {
    fn draw(&self, context: &mut Context<'_>) {
        let (x, y) = self.position();
        context.print(
            x,
            y,
            Line::from(Span::styled("@", Style::default().fg(Color::Yellow))),
        );
    }
}
