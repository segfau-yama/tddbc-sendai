use super::*;
use crate::view::position::Position;
use ratatui::{Terminal, backend::TestBackend};
use rust_molkky::Name;
use rust_molkky::core::{Field, FieldGenerator, Molkky, SimpleMolkky, Skittle};
use rust_molkky::simulation::{ArrayField, StandardFieldGenerator};
#[test]
fn 複数モデルから表示を組み立てても得点や配置は変わらない() {
    let model = vec![
        Team::new(Name::new("赤").unwrap()),
        Team::new(Name::new("青").unwrap()),
    ];
    let field = ArrayField::new(vec![Skittle::new(5), Skittle::new(12), Skittle::new(12)]);
    let stick = Stick::default();
    let mut layout = FieldLayout::new(field.skittles());
    layout.stick = layout.positions[&field.skittles()[0].id()];
    layout
        .positions
        .insert(field.skittles()[1].id(), Position::new(0, 3));
    layout
        .positions
        .insert(field.skittles()[2].id(), Position::new(2, 3));
    let view = compose_game(
        &SimpleMolkky,
        &model,
        &field,
        &stick,
        &layout,
        1,
        "テスト中",
    );
    assert_eq!(view.field.pins.len(), 3);
    assert_eq!(view.field.pins[1].number, 12);
    assert_eq!(view.field.pins[2].number, 12);
    assert_eq!(view.field.pins[1].position, Position::new(0, 3));
    assert_eq!(view.field.pins[2].position, Position::new(2, 3));
    assert_eq!(view.field.pins[0].number, 5);
    assert_eq!(
        view.field.pins[0].position,
        layout.positions[&field.skittles()[0].id()]
    );
    assert_eq!(view.field.pins[0].probability, 0.25);
    layout.stick = Position::new(i32::MAX, i32::MIN);
    assert_eq!(
        compose_game(&SimpleMolkky, &model, &field, &stick, &layout, 1, "")
            .field
            .pins[0]
            .probability,
        0.25
    );
    assert_eq!(model[0].score_board().value(), 0);
    assert!(view.teams[1].active);
    assert!(!view.teams[0].active);
}
#[test]
fn 補正と失格の説明はドメインの判定結果から作る() {
    let point = |n| {
        if n == 0 {
            SimpleMolkky.calc_score(&[])
        } else {
            SimpleMolkky.calc_score(&[Skittle::new(n)])
        }
    };
    let board = [12, 12, 12, 12, 1]
        .iter()
        .fold(ScoreBoard::new(), |b, &n| b.add(point(n)));
    let reset = board.add(point(2));
    assert!(
        compose_score(&reset)
            .notice
            .contains("50点を超過したため25点へ")
    );
    assert!(compose_score(&reset.add(point(12))).notice.is_empty());
    let missed = ScoreBoard::new().add(point(0)).add(point(0)).add(point(0));
    assert!(
        compose_score(&missed)
            .notice
            .contains("3回連続0点のため失格")
    );
    let team = Team::new(Name::new("赤").unwrap()).with_score_board(reset);
    assert!(throw_message(&team, 6, 49, point(2)).contains("49 → 25"));
}
#[test]
fn 三チームでも勝者の得点行が見える() {
    let miss = SimpleMolkky.calc_score(&[]);
    let disqualified = ScoreBoard::new().add(miss).add(miss).add(miss);
    let game = vec![
        Team::new(Name::new("WINNER").unwrap()),
        Team::new(Name::new("OUT_A").unwrap()).with_score_board(disqualified.clone()),
        Team::new(Name::new("OUT_B").unwrap()).with_score_board(disqualified),
    ];
    let field = StandardFieldGenerator.generate();
    let layout = FieldLayout::new(field.skittles());
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                compose_game(
                    &SimpleMolkky,
                    &game,
                    &field,
                    &Stick::default(),
                    &layout,
                    2,
                    "",
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
    assert!(text.contains("WINNERの勝利"));
    assert!(text.contains("勝WINNER0/50"));
}

#[test]
fn 表示合成でも渡されたルールの判定を利用する() {
    struct BlueWins;
    impl Molkky for BlueWins {
        fn judge_game(&self, _: &[Team]) -> GameStatus {
            GameStatus::Finished { winner: Some(1) }
        }
    }
    let teams = [
        Team::new(Name::new("赤").unwrap()),
        Team::new(Name::new("青").unwrap()),
    ];
    let field = StandardFieldGenerator.generate();
    let layout = FieldLayout::new(field.skittles());
    let view = compose_game(&BlueWins, &teams, &field, &Stick::default(), &layout, 0, "");
    assert!(view.banner.contains("青の勝利"));
    assert!(view.teams[1].winner);
    assert!(!view.teams[0].winner);
    assert_eq!(teams[1].score_board().value(), 0);
}
