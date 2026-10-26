use super::*;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::core::{Field, ScoreBoard, Skittle};

fn rendered(runtime: &Runtime) -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| runtime.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
        .split_whitespace()
        .collect()
}

fn almost_disqualified() -> ScoreBoard {
    let missed = calc_score(&[]);
    ScoreBoard::new().add(missed).add(missed)
}

#[test]
fn 残り一チームの勝利後は履歴と配置と棒を変えない() {
    let mut runtime = Runtime::new(vec!["赤".into(), "青".into()]);
    runtime.game = Molkky::new(vec![
        Team::new("赤").with_score_board(almost_disqualified()),
        Team::new("青"),
    ])
    .with_field(Field::new(vec![]));
    runtime.throw();
    assert!(runtime.game.teams()[0].score_board().is_disqualified());
    assert!(runtime.game.is_finished());
    assert_eq!(runtime.game.result(), Some(1));
    let history = runtime.game.teams()[0].score_board().history().to_vec();
    let pins = runtime.game.field().skittles().to_vec();
    let stick = *runtime.game.stick();
    runtime.throw();
    runtime.move_from(1, 0);
    assert_eq!(runtime.game.teams()[0].score_board().history(), history);
    assert_eq!(runtime.game.field().skittles(), pins);
    assert_eq!(*runtime.game.stick(), stick);
    assert!(rendered(&runtime).contains("青の勝利"));
}

#[test]
fn 失格チームを飛ばして二チーム以上なら継続する() {
    let mut runtime = Runtime::new(vec!["赤".into(), "青".into(), "緑".into()]);
    runtime.game = Molkky::new(vec![
        Team::new("赤").with_score_board(almost_disqualified()),
        Team::new("青"),
        Team::new("緑"),
    ])
    .with_field(Field::new(vec![]));
    runtime.throw();
    assert!(!runtime.game.is_finished());
    assert_eq!(runtime.active, 1);
    runtime.throw();
    assert_eq!(runtime.active, 2);
    runtime.throw();
    assert_eq!(runtime.active, 1);
    assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
    assert_eq!(runtime.game.teams()[1].score_board().history().len(), 1);
    assert_eq!(runtime.game.teams()[2].score_board().history().len(), 1);
}

#[test]
fn 一チーム開始なら三回連続0点で勝者なし終了する() {
    let mut runtime = Runtime::new(vec!["赤".into()]);
    runtime.game = runtime.game.with_field(Field::new(vec![]));
    assert!(!runtime.game.is_finished());
    for _ in 0..3 {
        runtime.throw();
    }
    assert!(runtime.game.is_finished());
    assert_eq!(runtime.game.result(), None);
    assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
    runtime.throw();
    assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
    assert!(rendered(&runtime).contains("全チームが失格"));
}

#[test]
fn 次のチームも直前の投てきで移動した共通フィールドと棒を使う() {
    let mut runtime = Runtime::new(vec!["赤".into(), "青".into()]);
    runtime.game = runtime.game.with_field(Field::new(vec![Skittle::new(5)]));
    runtime.throw();
    assert_eq!(runtime.game.teams()[0].score_board().value(), 5);
    assert_eq!(runtime.active, 1);
    let moved = runtime.game.field().skittles()[0].position();
    assert_ne!(moved, Position::new(0, 0));
    runtime.game = runtime.game.with_stick(Stick::new(moved));
    runtime.throw();
    assert_eq!(runtime.game.teams()[1].score_board().value(), 5);
    assert_ne!(runtime.game.field().skittles()[0].position(), moved);
    assert_eq!(runtime.game.stick().position(), moved);
    assert_eq!(runtime.game.teams()[0].score_board().history().len(), 1);
    assert_eq!(runtime.game.teams()[1].score_board().history().len(), 1);
}
