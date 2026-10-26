use super::CanvasDraw;
use crate::view::{position::Position, skittle::SkittleTui, stick::StickTui};
use ratatui::{Terminal, backend::TestBackend, widgets::canvas::Canvas};
#[test]
fn coreを構築せずピンと棒を同じtraitで描画する() {
    let pin = SkittleTui {
        number: 5,
        position: Position::new(0, 0),
        probability: 0.25,
    };
    let stick = StickTui {
        position: Position::new(0, -2),
    };
    let drawables: [&dyn CanvasDraw; 2] = [&pin, &stick];
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                Canvas::default()
                    .x_bounds([-3.0, 3.0])
                    .y_bounds([-3.0, 3.0])
                    .paint(|context| {
                        for item in drawables {
                            item.draw(context);
                        }
                    }),
                frame.area(),
            )
        })
        .unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(text.contains("05"));
    assert!(text.contains('@'));
}
