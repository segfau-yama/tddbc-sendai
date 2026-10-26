use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::{Field, Molkky, ScoreBoard, Skittle, Team, calc_score};

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
    let updated = original.record_point(calc_score(&[Skittle::new(11)]));
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
    let updated = original.relocate(&mut StdRng::seed_from_u64(42));
    assert_eq!(original.skittles(), before);
    for (old, new) in original.skittles().iter().zip(updated.skittles()) {
        assert_eq!(old.number(), new.number());
        assert_ne!(old.position(), new.position());
    }
}
