use super::*;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::core::{Field, ScoreBoard, Skittle, Stick};

fn rendered(runtime: &Controller) -> String {
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
    let mut runtime = Controller::new(vec!["赤".into(), "青".into()]);
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
    let layout = runtime.layout.clone();
    runtime.throw();
    runtime.move_from(1, 0);
    assert_eq!(runtime.game.teams()[0].score_board().history(), history);
    assert_eq!(runtime.game.field().skittles(), pins);
    assert_eq!(*runtime.game.stick(), stick);
    assert_eq!(runtime.layout, layout);
    assert!(rendered(&runtime).contains("青の勝利"));
}

#[test]
fn 失格チームを飛ばして二チーム以上なら継続する() {
    let mut runtime = Controller::new(vec!["赤".into(), "青".into(), "緑".into()]);
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
    let mut runtime = Controller::new(vec!["赤".into()]);
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
fn 次のチームも共通のピン集合と棒と表示を使う() {
    let mut controller = Controller::new(vec!["赤".into(), "青".into()]);
    controller.game = controller
        .game
        .with_field(Field::new(vec![Skittle::new(5)]))
        .with_stick(Stick::new(1.0));
    let original = controller.layout.positions[&5];
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.game.teams()[0].score_board().value(), 5);
    assert_eq!(controller.active, 1);
    let moved = controller.layout.positions[&5];
    assert_ne!(moved, original);
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.game.teams()[1].score_board().value(), 5);
    assert_ne!(controller.layout.positions[&5], moved);
    assert_eq!(controller.game.field().skittles(), [Skittle::new(5)]);
    assert_eq!(controller.game.stick().fall_probability(), 1.0);
    assert_eq!(controller.game.teams()[0].score_board().history().len(), 1);
    assert_eq!(controller.game.teams()[1].score_board().history().len(), 1);
}
#[test]
fn 棒の表示をピンへ重ねても確率と投てき結果は変わらない() {
    use rand::{SeedableRng, rngs::StdRng};
    let mut a = Controller::new(vec!["赤".into(), "青".into()]);
    let mut b = Controller::new(vec!["赤".into(), "青".into()]);
    a.layout.stick = a.layout.positions[&5];
    b.move_from(100, -100);
    a.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    b.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    assert_eq!(a.game.teams()[0].score_board().value(), 0);
    let (mut first, mut second) = (StdRng::seed_from_u64(71), StdRng::seed_from_u64(71));
    for _ in 0..10 {
        a.throw_with_rng(&mut first);
        b.throw_with_rng(&mut second);
        assert_eq!(a.game.status(), b.game.status());
        assert_eq!(a.active, b.active);
        for (left, right) in a.game.teams().iter().zip(b.game.teams()) {
            assert_eq!(left.score_board().history(), right.score_board().history());
        }
    }
    assert_eq!(a.game.stick().fall_probability(), 0.25);
}
