use super::{draw::CanvasDraw, position::Position};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Row, canvas::Context},
};

pub(crate) struct SkittleTui {
    pub(crate) number: i32,
    pub(crate) position: Position,
    pub(crate) probability: f64,
}
impl SkittleTui {
    pub(crate) fn label(&self) -> String {
        format!("{:02}", self.number)
    }
    pub(crate) fn position_row(&self) -> Row<'static> {
        Row::new([
            self.label(),
            self.position.table_label(),
            format!("{:>5.1}%", self.probability * 100.0),
        ])
    }
    pub(crate) fn draw_label(context: &mut Context<'_>, position: Position, label: String) {
        let (x, y) = position.canvas();
        context.print(
            x,
            y,
            Line::from(Span::styled(label, Style::default().fg(Color::Cyan))),
        );
    }
}
impl CanvasDraw for SkittleTui {
    fn draw(&self, context: &mut Context<'_>) {
        Self::draw_label(context, self.position, self.label());
    }
}
