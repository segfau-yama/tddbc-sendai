use std::collections::BTreeMap;

use super::{draw::CanvasDraw, position::Position, skittle::SkittleTui, stick::StickTui};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Color,
    widgets::{
        Block, Table, Widget,
        canvas::{Canvas, Line as CanvasLine},
    },
};

pub(crate) struct FieldTui {
    pub(crate) pins: Vec<SkittleTui>,
    pub(crate) stick: StickTui,
}
impl FieldTui {
    pub(crate) fn positions_table(&self) -> Table<'static> {
        Table::new(
            self.pins.iter().map(SkittleTui::position_row),
            [
                Constraint::Length(3),
                Constraint::Min(9),
                Constraint::Length(6),
            ],
        )
        .block(Block::bordered().title("ピン位置 (x,y) / 確率"))
    }
}

impl Widget for FieldTui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let stick = self.stick;
        let (from_x, from_y) = stick.position();
        let (mut left, mut right, mut bottom, mut top) = (from_x, from_x, from_y, from_y);
        let mut groups: BTreeMap<(i32, i32), Vec<&SkittleTui>> = BTreeMap::new();
        for pin in &self.pins {
            let position = pin.position;
            let (x, y) = position.canvas();
            left = left.min(x);
            right = right.max(x);
            bottom = bottom.min(y);
            top = top.max(y);
            groups
                .entry((position.x(), position.y()))
                .or_default()
                .push(pin);
        }
        let canvas = Canvas::default()
            .block(Block::bordered().title("盤面  番号=ピン  @=棒の表示位置"))
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
                            .map(|pin| pin.label())
                            .collect::<Vec<_>>()
                            .join("/");
                        SkittleTui::draw_label(context, Position::new(x, y), label);
                    }
                }
                stick.draw(context);
            });
        canvas.render(area, buf);
    }
}

#[cfg(test)]
#[path = "../../tests/view/field.rs"]
mod tests;
