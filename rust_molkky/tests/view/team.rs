use super::TeamTui;
use crate::view::score::ScoreBoardTui;
use ratatui::{Terminal, backend::TestBackend, layout::Constraint, widgets::Table};
#[test]
fn 得点が零でも勝者と失格と操作対象の行を区別する() {
    let teams: Vec<_> = [
        ("青", 0, false, true, false),
        ("赤", 0, false, false, true),
        ("緑", 5, true, false, false),
    ]
    .into_iter()
    .map(|(name, value, active, winner, disqualified)| TeamTui {
        name: name.into(),
        board: ScoreBoardTui {
            value,
            target: 50,
            history: vec![],
            notice: String::new(),
        },
        active,
        winner,
        disqualified,
    })
    .collect();
    let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                Table::new(
                    teams.iter().map(TeamTui::row),
                    [
                        Constraint::Length(5),
                        Constraint::Length(10),
                        Constraint::Length(10),
                        Constraint::Length(5),
                        Constraint::Min(10),
                    ],
                ),
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
        .collect::<String>()
        .split_whitespace()
        .collect();
    assert!(text.contains("勝青0/50"));
    assert!(text.contains("失格赤0/50"));
    assert!(text.contains("▶緑5/50"));
}
