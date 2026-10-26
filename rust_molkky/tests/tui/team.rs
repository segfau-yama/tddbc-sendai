use super::TeamTui;
use ratatui::{Terminal, backend::TestBackend, layout::Constraint, widgets::Table};
use rust_molkky::core::{ScoreBoard, Skittle, Team, calc_score};

#[test]
fn 得点が0でも勝者と失格と操作対象の行を区別する() {
    let missed = calc_score(&[]);
    let winner = Team::new("青");
    let disqualified =
        Team::new("赤").with_score_board(ScoreBoard::new().add(missed).add(missed).add(missed));
    let active =
        Team::new("緑").with_score_board(ScoreBoard::new().add(calc_score(&[Skittle::new(5)])));
    let rows = vec![
        TeamTui::new(&winner).row(false, true),
        TeamTui::new(&disqualified).row(false, false),
        TeamTui::new(&active).row(true, false),
    ];
    let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
    terminal
        .draw(|frame| {
            let table = Table::new(
                rows,
                [
                    Constraint::Length(5),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(5),
                    Constraint::Min(10),
                ],
            );
            frame.render_widget(table, frame.area());
        })
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
    assert!(text.contains("勝青0/50"));
    assert!(text.contains("失格赤0/50"));
    assert!(text.contains("▶緑5/50"));
}
