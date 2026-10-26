use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::{
    Field, Molkky, Position, ScoreBoard, Skittle, Stick, Team, calc_score, relocate_skittles,
};

#[test]
fn ゲームが共通の配置と棒を所有し更新しても元の状態を保持する() {
    let pins = vec![Skittle::new(5).with_position(Position::new(3, 4))];
    let original = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    let placed = original.with_field(Field::new(pins.clone()));
    let aimed = placed.with_stick(Stick::new(Position::new(7, 8)));
    let scored = aimed.record_point(0, calc_score(&[Skittle::new(5)]));
    assert_eq!(original.field().skittles().len(), 12);
    assert_eq!(placed.field().skittles(), pins);
    assert_eq!(aimed.field().skittles(), pins);
    assert_eq!(scored.field().skittles(), pins);
    assert_eq!(placed.stick().position(), Position::new(0, 0));
    assert_eq!(aimed.stick().position(), Position::new(7, 8));
    assert_eq!(scored.stick().position(), Position::new(7, 8));
    assert_eq!(original.stick().position(), Position::new(0, 0));
    assert_eq!(placed.teams()[0].score_board().value(), 0);
    assert_eq!(scored.teams()[0].score_board().value(), 5);
    let reconfigured = scored.with_field(Field::new(vec![]));
    assert_eq!(scored.field().skittles(), pins);
    assert!(reconfigured.field().skittles().is_empty());
    assert_eq!(reconfigured.teams()[0].score_board().value(), 5);
    assert_eq!(reconfigured.stick().position(), Position::new(7, 8));
}

#[test]
fn 点数を追加しても元の得点表の履歴と得点は変わらない() {
    let five = calc_score(&[Skittle::new(5)]);
    let eleven = calc_score(&[Skittle::new(11)]);
    let original = ScoreBoard::new().add(five);
    let updated = original.add(eleven);
    assert_eq!(original.history(), &[five]);
    assert_eq!(original.value(), 5);
    assert_eq!(updated.history(), &[five, eleven]);
    assert_eq!(updated.value(), 16);
}

#[test]
fn チームへの記録でも元のチームは変わらない() {
    let original = Team::new("赤");
    let updated =
        original.with_score_board(original.score_board().add(calc_score(&[Skittle::new(11)])));
    assert_eq!(original.score_board().value(), 0);
    assert!(original.score_board().history().is_empty());
    assert_eq!(updated.score_board().value(), 11);
    assert_eq!(updated.name(), original.name());
}

#[test]
fn ゲームへの記録でも元のゲームと他チームは変わらない() {
    let original = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
    let updated = original.record_point(0, calc_score(&[Skittle::new(11)]));
    assert_eq!(original.teams()[0].score_board().value(), 0);
    assert_eq!(original.teams()[1].score_board().value(), 0);
    assert_eq!(updated.teams()[0].score_board().value(), 11);
    assert_eq!(updated.teams()[1].score_board().value(), 0);
}

#[test]
fn ピンの移動でも元の配置は変わらない() {
    let original = Field::default();
    let before = original.skittles().to_vec();
    let updated = relocate_skittles(&original, &mut StdRng::seed_from_u64(42));
    assert_eq!(original.skittles(), before);
    for (old, new) in original.skittles().iter().zip(updated.skittles()) {
        assert_eq!(old.number(), new.number());
        assert_ne!(old.position(), new.position());
    }
}
