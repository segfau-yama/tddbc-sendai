use std::collections::BTreeMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Table, Widget,
        canvas::{Canvas, Line as CanvasLine},
    },
};
use rust_molkky::core::{Field, Stick};

use super::{draw::CanvasDraw, position::PositionTui, skittle::SkittleTui, stick::StickTui};

pub(super) struct FieldTui<'a> {
    field: &'a Field,
    stick: &'a Stick,
}

impl<'a> FieldTui<'a> {
    pub(super) fn new(field: &'a Field, stick: &'a Stick) -> Self {
        Self { field, stick }
    }

    pub(super) fn positions_table(&self) -> Table<'_> {
        let rows = self
            .field
            .skittles()
            .iter()
            .map(|pin| SkittleTui::new(pin).position_row(self.stick));
        Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Min(9),
                Constraint::Length(6),
            ],
        )
        .block(Block::bordered().title("ピン位置 (x,y) / 確率"))
    }
}

impl Widget for FieldTui<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let stick = StickTui::new(self.stick);
        let (from_x, from_y) = stick.position();
        let (mut left, mut right, mut bottom, mut top) = (from_x, from_x, from_y, from_y);
        let mut groups: BTreeMap<(i32, i32), Vec<SkittleTui<'_>>> = BTreeMap::new();
        for pin in self.field.skittles() {
            let position = pin.position();
            let (x, y) = PositionTui::new(&position).canvas();
            left = left.min(x);
            right = right.max(x);
            bottom = bottom.min(y);
            top = top.max(y);
            groups
                .entry((position.x(), position.y()))
                .or_default()
                .push(SkittleTui::new(pin));
        }
        let canvas = Canvas::default()
            .block(Block::bordered().title("盤面  番号=ピン  @=投てき位置"))
            .x_bounds([left - 3.0, right + 3.0])
            .y_bounds([bottom - 2.0, top + 2.0])
            .paint(|context| {
                context.draw(&CanvasLine {
                    x1: left - 3.0,
                    y1: from_y,
                    x2: right + 3.0,
                    y2: from_y,
                    color: Color::DarkGray,
                });
                context.draw(&CanvasLine {
                    x1: from_x,
                    y1: bottom - 2.0,
                    x2: from_x,
                    y2: top + 2.0,
                    color: Color::DarkGray,
                });
                for (&(x, y), pins) in &groups {
                    if let [pin] = pins.as_slice() {
                        pin.draw(context);
                    } else {
                        // 同じ場所のピンを上書きせず、番号をまとめて描画する。
                        let label = pins
                            .iter()
                            .map(SkittleTui::label)
                            .collect::<Vec<_>>()
                            .join("/");
                        context.print(
                            f64::from(x),
                            f64::from(y),
                            Line::from(Span::styled(label, Style::default().fg(Color::Cyan))),
                        );
                    }
                }
                stick.draw(context);
            });
        canvas.render(area, buf);
    }
}

#[cfg(test)]
#[path = "../../tests/tui/field.rs"]
mod tests;
