use super::*;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::Name;
use rust_molkky::ScoreState;
use rust_molkky::core::{Field, ScoreBoard, Skittle};

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
    let missed = SimpleMolkky.calc_score(&[]);
    ScoreBoard::default().add(missed).add(missed)
}

#[test]
fn 残り一チームの勝利後は履歴と配置と棒を変えない() {
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    runtime.teams = vec![
        Team::new(Name::new("赤").unwrap()).with_score_board(almost_disqualified()),
        Team::new(Name::new("青").unwrap()),
    ];
    runtime.teams[0] = runtime.teams[0].with_turn(true);
    runtime.field = ArrayField::new(vec![]);
    runtime.layout = FieldLayout::new(runtime.field.skittles());
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert!(matches!(
        runtime.teams[0].score_board().state(),
        ScoreState::Disqualified
    ));
    assert!(SimpleMolkky.next_team(&runtime.teams).is_none());
    assert_eq!(
        winning_team(&SimpleMolkky, &runtime.teams).unwrap().id(),
        runtime.teams[1].id()
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
    runtime.teams[0] = runtime.teams[0].with_turn(true);
    runtime.field = ArrayField::new(vec![]);
    runtime.layout = FieldLayout::new(runtime.field.skittles());
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert!(!SimpleMolkky.next_team(&runtime.teams).is_none());
    assert_eq!(
        SimpleMolkky.next_team(&runtime.teams).unwrap().id(),
        runtime.teams[1].id()
    );
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(
        SimpleMolkky.next_team(&runtime.teams).unwrap().id(),
        runtime.teams[2].id()
    );
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(
        SimpleMolkky.next_team(&runtime.teams).unwrap().id(),
        runtime.teams[1].id()
    );
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    assert_eq!(runtime.teams[1].score_board().history().len(), 1);
    assert_eq!(runtime.teams[2].score_board().history().len(), 1);
}

#[test]
fn 一チーム開始なら三回連続0点で勝者なし終了する() {
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap()]);
    runtime.teams[0] = runtime.teams[0].with_turn(true);
    runtime.field = ArrayField::new(vec![]);
    runtime.layout = FieldLayout::new(runtime.field.skittles());
    assert!(!SimpleMolkky.next_team(&runtime.teams).is_none());
    for _ in 0..3 {
        runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    }
    assert!(SimpleMolkky.next_team(&runtime.teams).is_none());
    assert!(SimpleMolkky.judge_game(&runtime.teams).is_none());
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.teams[0].score_board().history().len(), 3);
    assert!(rendered(&runtime).contains("全チームが失格"));
}

#[test]
fn 次のチームも共通のピン集合と棒と表示を使う() {
    let mut controller = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    controller.field = ArrayField::new(vec![Skittle::new(5)]);
    controller.layout = FieldLayout::new(controller.field.skittles());
    controller.stick = Stick::new(1.0);
    let original = controller.layout.positions[&controller.field.skittles()[0].id()];
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.teams[0].score_board().total(), 5);
    assert_eq!(
        SimpleMolkky.next_team(&controller.teams).unwrap().id(),
        controller.teams[1].id()
    );
    let moved = controller.layout.positions[&controller.field.skittles()[0].id()];
    assert_ne!(moved, original);
    controller.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(controller.teams[1].score_board().total(), 5);
    assert_ne!(
        controller.layout.positions[&controller.field.skittles()[0].id()],
        moved
    );
    assert_eq!(
        controller
            .field
            .skittles()
            .iter()
            .map(Skittle::number)
            .collect::<Vec<_>>(),
        [5]
    );
    assert_eq!(controller.stick.fall_probability(), 1.0);
    assert_eq!(controller.teams[0].score_board().history().len(), 1);
    assert_eq!(controller.teams[1].score_board().history().len(), 1);
}
#[test]
fn 棒の表示をピンへ重ねても確率と投てき結果は変わらない() {
    use rand::{SeedableRng, rngs::StdRng};
    let mut a = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    let mut b = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    a.layout.stick = a.layout.positions[&a.field.skittles()[4].id()];
    b.move_from(100, -100);
    a.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    b.throw_with_rng(&mut crate::test_support::ConstantRng(u64::MAX));
    assert_eq!(a.teams[0].score_board().total(), 0);
    let (mut first, mut second) = (StdRng::seed_from_u64(71), StdRng::seed_from_u64(71));
    for _ in 0..10 {
        a.throw_with_rng(&mut first);
        b.throw_with_rng(&mut second);
        assert_eq!(
            winning_team(&SimpleMolkky, &a.teams).map(|team| team.name().as_str()),
            winning_team(&SimpleMolkky, &b.teams).map(|team| team.name().as_str())
        );
        assert_eq!(
            SimpleMolkky
                .next_team(&a.teams)
                .map(|team| team.name().as_str()),
            SimpleMolkky
                .next_team(&b.teams)
                .map(|team| team.name().as_str())
        );
        for (left, right) in a.teams.iter().zip(&b.teams) {
            assert_eq!(left.score_board().history(), right.score_board().history());
        }
    }
    assert_eq!(a.stick.fall_probability(), 0.25);
}

#[test]
fn 五十点勝利後は手番を外し使用側で追加投てきを停止する() {
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    let history = [12, 12, 12, 12, 1]
        .map(|n| SimpleMolkky.calc_score(&[Skittle::new(n)]))
        .to_vec();
    runtime.teams[0] = runtime.teams[0].with_score_board(ScoreBoard::new(history));
    runtime.field = ArrayField::new(vec![Skittle::new(1)]);
    runtime.layout = FieldLayout::new(runtime.field.skittles());
    runtime.stick = Stick::new(1.0);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.teams[0].score_board().state(), ScoreState::Won);
    assert_eq!(
        SimpleMolkky.judge_game(&runtime.teams).unwrap().id(),
        runtime.teams[0].id()
    );
    assert!(runtime.teams.iter().all(|team| !team.turn()));
    let layout = runtime.layout.clone();
    let history = runtime.teams[0].score_board().history().to_vec();
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    runtime.move_from(1, 0);
    assert_eq!(runtime.layout, layout);
    assert_eq!(runtime.teams[0].score_board().history(), history);
    assert!(rendered(&runtime).contains("赤の勝利"));
}

#[test]
fn 使用側で超過補正を表示し次の手番を設定する() {
    let mut runtime = Controller::new(vec![Name::new("赤").unwrap(), Name::new("青").unwrap()]);
    let history = [12, 12, 12, 12, 1]
        .map(|n| SimpleMolkky.calc_score(&[Skittle::new(n)]))
        .to_vec();
    runtime.teams[0] = runtime.teams[0].with_score_board(ScoreBoard::new(history));
    runtime.field = ArrayField::new(vec![Skittle::new(2)]);
    runtime.layout = FieldLayout::new(runtime.field.skittles());
    runtime.stick = Stick::new(1.0);
    runtime.throw_with_rng(&mut crate::test_support::ConstantRng(0));
    assert_eq!(runtime.teams[0].score_board().state(), ScoreState::Reset);
    assert_eq!(runtime.teams[0].score_board().total(), 25);
    assert_eq!(
        SimpleMolkky.next_team(&runtime.teams).unwrap().id(),
        runtime.teams[1].id()
    );
    assert_eq!(runtime.teams.iter().filter(|team| team.turn()).count(), 1);
    assert!(rendered(&runtime).contains("25点へ"));
}
