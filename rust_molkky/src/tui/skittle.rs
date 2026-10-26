use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Row, canvas::Context},
};
use rust_molkky::core::{Skittle, Stick, fall_probability};

use super::position::PositionTui;

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
        let (x, y) = PositionTui::new(&position).canvas();
        context.print(
            x,
            y,
            Line::from(Span::styled(self.label(), Style::default().fg(Color::Cyan))),
        );
    }

    pub(super) fn position_row(&self, stick: &Stick) -> Row<'static> {
        Row::new([
            self.label(),
            PositionTui::new(&self.skittle.position()).table_label(),
            format!("{:>5.1}%", fall_probability(stick, self.skittle) * 100.0),
        ])
    }
}
