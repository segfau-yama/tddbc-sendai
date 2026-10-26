use super::{FieldLayout, FieldTui};
use crate::view::{position::Position, skittle::SkittleTui, stick::StickTui};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};
#[test]
fn 同じ位置の番号をまとめ棒と確率も表示する() {
    let pins = [(1, 0, 0), (2, 0, 0), (3, 2, 2)]
        .into_iter()
        .map(|(number, x, y)| SkittleTui {
            number,
            position: Position::new(x, y),
            probability: 0.25,
        })
        .collect();
    let view = FieldTui {
        pins,
        stick: StickTui {
            position: Position::new(0, -3),
        },
    };
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(view.positions_table(), Rect::new(55, 0, 25, 24));
            frame.render_widget(view, Rect::new(0, 0, 55, 24));
        })
        .unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(text.contains("01/02"));
    assert!(text.contains("03"));
    assert!(text.contains('@'));
    assert!(text.contains("25.0%"));
}
#[test]
fn 初期表示は投てき側から二本三本四本三本の順に並ぶ() {
    let layout = FieldLayout::default();
    for (y, expected) in [
        (3, vec![1, 2]),
        (4, vec![3, 10, 4]),
        (5, vec![5, 11, 12, 6]),
        (6, vec![7, 9, 8]),
    ] {
        let mut row: Vec<_> = layout
            .positions
            .iter()
            .filter(|(_, p)| p.y() == y)
            .collect();
        row.sort_by_key(|(_, p)| p.x());
        assert_eq!(row.iter().map(|(n, _)| **n).collect::<Vec<_>>(), expected);
    }
    assert_eq!(layout.positions.len(), 12);
}
