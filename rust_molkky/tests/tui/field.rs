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
            frame.render_widget(FieldTui::new(&field, &stick), frame.area());
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
