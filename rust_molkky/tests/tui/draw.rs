use super::CanvasDraw;
use crate::tui::{skittle::SkittleTui, stick::StickTui};
use ratatui::{Terminal, backend::TestBackend, widgets::canvas::Canvas};
use rust_molkky::core::{Position, Skittle, Stick};

#[test]
fn ピンと棒を同じtrait経由で描画する() {
    let skittle = Skittle::new(5);
    let stick = Stick::new(Position::new(0, -2));
    let pin_view = SkittleTui::new(&skittle);
    let stick_view = StickTui::new(&stick);
    let drawables: [&dyn CanvasDraw; 2] = [&pin_view, &stick_view];
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    terminal
        .draw(|frame| {
            let canvas = Canvas::default()
                .x_bounds([-3.0, 3.0])
                .y_bounds([-3.0, 3.0])
                .paint(|context| {
                    for drawable in drawables {
                        drawable.draw(context);
                    }
                });
            frame.render_widget(canvas, frame.area());
        })
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("05"));
    assert!(rendered.contains('@'));
}
