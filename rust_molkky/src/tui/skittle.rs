use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Context,
};
use rust_molkky::core::Skittle;

pub(super) struct SkittleTui<'a> {
    skittle: &'a Skittle,
}

impl<'a> SkittleTui<'a> {
    pub(super) fn new(skittle: &'a Skittle) -> Self {
        Self { skittle }
    }

    pub(super) fn label(&self) -> String {
        format!("{:02}", self.skittle.number())
    }

    pub(super) fn draw(&self, context: &mut Context<'_>) {
        let position = self.skittle.position();
        context.print(
            f64::from(position.x()),
            f64::from(position.y()),
            Line::from(Span::styled(self.label(), Style::default().fg(Color::Cyan))),
        );
    }
}
