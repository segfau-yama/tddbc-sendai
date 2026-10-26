use super::MolkkyTui;
use crate::view::{field::FieldTui, position::Position, stick::StickTui};
use ratatui::{Terminal, backend::TestBackend};
#[test]
fn 小さい端末では表示値だけで必要サイズを案内する() {
    let view = MolkkyTui {
        teams: vec![],
        field: FieldTui {
            pins: vec![],
            stick: StickTui {
                position: Position::new(0, 0),
            },
        },
        active: 0,
        banner: String::new(),
        message: "テスト中".into(),
        help: String::new(),
    };
    let mut terminal = Terminal::new(TestBackend::new(70, 20)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(view, frame.area()))
        .unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
        .split_whitespace()
        .collect();
    assert!(text.contains("端末を72列×24行以上に広げてください。"));
    assert!(!text.contains("テスト中"));
}
