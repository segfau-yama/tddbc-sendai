use super::*;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::Name;
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
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    runtime.teams = vec![
        Team::new(Name::new("赤").unwrap()).with_score_board(almost_disqualified()),
        Team::new(Name::new("青").unwrap()),
    ];
    runtime.field = Field::new(vec![]);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert!(runtime.teams[0].score_board().is_disqualified());
    assert!(matches!(
        judge_game(&runtime.teams),
        GameStatus::Finished { .. }
    ));
    assert_eq!(
        judge_game(&runtime.teams),
        GameStatus::Finished { winner: Some(1) }
    );
    let history = runtime.teams[0].score_board().history().to_vec();
    let pins = runtime.field.skittles().to_vec();
    let stick = runtime.stick;
    let layout = runtime.layout.clone();
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    runtime.move_from(1, 0);
    assert_eq!(runtime.teams[0].score_board().history(), history);
    assert_eq!(runtime.field.skittles(), pins);
    assert_eq!(runtime.stick, stick);
    assert_eq!(runtime.layout, layout);
    assert!(rendered(&runtime).contains("青の勝利"));
}

#[test]
fn 失格チームを飛ばして二チーム以上なら継続する() {
    let mut runtime = Controller::new(vec![
        Name::new("赤").unwrap(),
        Name::new("青").unwrap(),
        Name::new("緑").unwrap(),
    ]);
    runtime.teams = vec![
        Team::new(Name::new("赤").unwrap()).with_score_board(almost_disqualified()),
        Team::new(Name::new("青").unwrap()),
        Team::new(Name::new("緑").unwrap()),
    ];
    runtime.field = Field::new(vec![]);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert!(!matches!(
        judge_game(&runtime.teams),
        GameStatus::Finished { .. }
    ));
    assert_eq!(runtime.active, 1);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.active, 2);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.active, 1);
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    assert_eq!(runtime.teams[1].score_board().history().len(), 1);
    assert_eq!(runtime.teams[2].score_board().history().len(), 1);
}

#[test]
fn 一チーム開始なら三回連続0点で勝者なし終了する() {
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap()]);
    runtime.field = Field::new(vec![]);
    assert!(!matches!(
        judge_game(&runtime.teams),
        GameStatus::Finished { .. }
    ));
    for _ in 0..3 {
        runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    }
    assert!(matches!(
        judge_game(&runtime.teams),
        GameStatus::Finished { .. }
    ));
    assert_eq!(
        judge_game(&runtime.teams),
        GameStatus::Finished { winner: None }
    );
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    assert!(rendered(&runtime).contains("全チームが失格"));
}

#[test]
fn 次のチームも共通のピン集合と棒と表示を使う() {
    let mut controller = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    controller.field = Field::new(vec![Skittle::new(5)]);
    controller.stick = Stick::new(1.0);
    let original = controller.layout.positions[&5];
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.teams[0].score_board().value(), 5);
    assert_eq!(controller.active, 1);
    let moved = controller.layout.positions[&5];
    assert_ne!(moved, original);
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.teams[1].score_board().value(), 5);
    assert_ne!(controller.layout.positions[&5], moved);
    assert_eq!(controller.field.skittles(), [Skittle::new(5)]);
    assert_eq!(controller.stick.fall_probability(), 1.0);
    assert_eq!(controller.teams[0].score_board().history().len(), 1);
    assert_eq!(controller.teams[1].score_board().history().len(), 1);
}
#[test]
fn 棒の表示をピンへ重ねても確率と投てき結果は変わらない() {
    use rand::{SeedableRng, rngs::StdRng};
    let mut a = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    let mut b = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    a.layout.stick = a.layout.positions[&5];
    b.move_from(100, -100);
    a.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    b.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    assert_eq!(a.teams[0].score_board().value(), 0);
    let (mut first, mut second) = (StdRng::seed_from_u64(71), StdRng::seed_from_u64(71));
    for _ in 0..10 {
        a.throw_with_rng(&mut first);
        b.throw_with_rng(&mut second);
        assert_eq!(judge_game(&a.teams), judge_game(&b.teams));
        assert_eq!(a.active, b.active);
        for (left, right) in a.teams.iter().zip(&b.teams) {
            assert_eq!(left.score_board().history(), right.score_board().history());
        }
    }
    assert_eq!(a.stick.fall_probability(), 0.25);
}
