use super::MolkkyTui;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::core::{Molkky, Team};

#[test]
fn 小さい端末では盤面の代わりに必要サイズを表示する() {
    let game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    let mut terminal = Terminal::new(TestBackend::new(70, 20)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(MolkkyTui::new(&game, 0, "テスト中"), frame.area()))
        .unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
        .split_whitespace()
        .collect::<String>();
    assert!(text.contains("端末を72列×24行以上に広げてください。"));
    assert!(!text.contains("テスト中"));
}
