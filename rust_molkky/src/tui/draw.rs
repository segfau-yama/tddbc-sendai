use ratatui::widgets::canvas::Context;

pub(super) trait CanvasDraw {
    fn draw(&self, context: &mut Context<'_>);
}

#[cfg(test)]
#[path = "../../tests/tui/draw.rs"]
mod tests;
