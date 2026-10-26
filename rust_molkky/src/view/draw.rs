use ratatui::widgets::canvas::Context;

pub(crate) trait CanvasDraw {
    fn draw(&self, context: &mut Context<'_>);
}

#[cfg(test)]
#[path = "../../tests/view/draw.rs"]
mod tests;
