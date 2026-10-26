use std::collections::BTreeMap;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block,
        canvas::{Canvas, Line as CanvasLine},
    },
};
use rust_molkky::core::{Field, Stick};

use super::{skittle::SkittleTui, stick::StickTui};

pub(super) struct FieldTui<'a> {
    field: &'a Field,
    stick: &'a Stick,
}

impl<'a> FieldTui<'a> {
    pub(super) fn new(field: &'a Field, stick: &'a Stick) -> Self {
        Self { field, stick }
    }

    pub(super) fn draw(&self, frame: &mut Frame, area: Rect) {
        let stick = StickTui::new(self.stick);
        let (from_x, from_y) = stick.position();
        let (mut left, mut right, mut bottom, mut top) = (from_x, from_x, from_y, from_y);
        let mut groups: BTreeMap<(i32, i32), Vec<SkittleTui<'_>>> = BTreeMap::new();
        for pin in self.field.skittles() {
            let position = pin.position();
            let (x, y) = (f64::from(position.x()), f64::from(position.y()));
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
        frame.render_widget(canvas, area);
    }
}

#[cfg(test)]
mod tests {
    use super::FieldTui;
    use ratatui::{Terminal, backend::TestBackend};
    use rust_molkky::core::{Field, Position, Skittle, Stick};

    #[test]
    fn 同じ位置のピン番号をまとめ棒も描画する() {
        let field = Field::new(vec![
            Skittle::new(1),
            Skittle::new(2),
            Skittle::new(3).with_position(Position::new(2, 2)),
        ]);
        let stick = Stick::new(Position::new(0, -3));
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| {
                FieldTui::new(&field, &stick).draw(frame, frame.area());
            })
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("01/02"));
        assert!(rendered.contains("03"));
        assert!(rendered.contains('@'));
    }
}
