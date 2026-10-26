use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::{Field, Position, Skittle, Stick, calc_score, relocate_skittles, throw_stick};

mod support;
use support::ConstantRng;

#[test]
fn すべてのピンが倒れる乱数なら12点になる() {
    let field = Field::default();
    let knocked_over = throw_stick(
        &Stick::new(Position::new(0, 0)),
        field.skittles(),
        &mut ConstantRng(0),
    );
    assert_eq!(calc_score(&knocked_over).value(), 12);
}

#[test]
fn どのピンも倒れない乱数なら0点になる() {
    let field = Field::default();
    let knocked_over = throw_stick(
        &Stick::new(Position::new(0, 0)),
        field.skittles(),
        &mut ConstantRng(u64::MAX),
    );
    assert_eq!(calc_score(&knocked_over).value(), 0);
}

#[test]
fn 同じシードなら同じ投てきの点数列と配置を再現できる() {
    let mut first = StdRng::seed_from_u64(20261024);
    let mut second = StdRng::seed_from_u64(20261024);
    let mut first_field = Field::default();
    let mut second_field = Field::default();
    let stick = Stick::new(Position::new(0, 0));
    for _ in 0..100 {
        let first_knocked = throw_stick(&stick, first_field.skittles(), &mut first);
        let second_knocked = throw_stick(&stick, second_field.skittles(), &mut second);
        assert_eq!(first_knocked, second_knocked);
        let points = calc_score(&first_knocked).value();
        assert!((0..=12).contains(&points));
        assert_eq!(points, calc_score(&second_knocked).value());
        first_field = relocate_skittles(&first_field, &mut first);
        second_field = relocate_skittles(&second_field, &mut second);
        assert_eq!(first_field.skittles(), second_field.skittles());
    }
}

#[test]
fn 同じ乱数でも近いスキットルだけが倒れる() {
    let field = Field::new(vec![
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(0, 4)),
    ]);
    let knocked_over = throw_stick(
        &Stick::new(Position::new(0, 0)),
        field.skittles(),
        &mut ConstantRng(1 << 62),
    );
    assert_eq!(calc_score(&knocked_over).value(), 5);
}

#[test]
fn 投てき場所を変えると倒れるスキットルが変わる() {
    let field = Field::new(vec![
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(0, 4)),
    ]);
    let near_five = throw_stick(
        &Stick::new(Position::new(0, 0)),
        field.skittles(),
        &mut ConstantRng(1 << 62),
    );
    let near_eight = throw_stick(
        &Stick::new(Position::new(0, 4)),
        field.skittles(),
        &mut ConstantRng(1 << 62),
    );
    assert_eq!(calc_score(&near_five).value(), 5);
    assert_eq!(calc_score(&near_eight).value(), 8);
}

#[test]
fn 投てき後に全ピンを移動しても番号と本数と点数は保たれる() {
    let mut field = Field::default();
    let numbers: Vec<_> = field.skittles().iter().map(Skittle::number).collect();
    let mut rng = ConstantRng(0);
    for _ in 0..3 {
        let before = field.skittles().to_vec();
        let knocked_over =
            throw_stick(&Stick::new(Position::new(0, 0)), field.skittles(), &mut rng);
        let next = relocate_skittles(&field, &mut rng);
        assert_eq!(field.skittles(), before);
        assert_eq!(calc_score(&knocked_over).value(), 12);
        assert_eq!(next.skittles().len(), 12);
        assert_eq!(
            next.skittles()
                .iter()
                .map(Skittle::number)
                .collect::<Vec<_>>(),
            numbers
        );
        for (old, new) in before.iter().zip(next.skittles()) {
            assert_ne!(old.position(), new.position());
            assert!((i64::from(old.position().x()) - i64::from(new.position().x())).abs() <= 1);
            assert!((i64::from(old.position().y()) - i64::from(new.position().y())).abs() <= 1);
        }
        field = next;
    }
}

#[test]
fn 空のフィールドは何も移動せず0点になる() {
    let field = Field::new(vec![]);
    let mut rng = ConstantRng(0);
    let knocked_over = throw_stick(&Stick::new(Position::new(0, 0)), field.skittles(), &mut rng);
    let updated = relocate_skittles(&field, &mut rng);
    assert_eq!(calc_score(&knocked_over).value(), 0);
    assert!(field.skittles().is_empty());
    assert!(updated.skittles().is_empty());
}

#[test]
fn 倒れなかったピンも移動し移動前の投てきの点数は変わらない() {
    let original = Field::new(vec![Skittle::new(5).with_position(Position::new(0, 3))]);
    let mut rng = ConstantRng(u64::MAX);
    let missed = throw_stick(
        &Stick::new(Position::new(0, 0)),
        original.skittles(),
        &mut rng,
    );
    assert_eq!(calc_score(&missed).value(), 0);
    let relocated = relocate_skittles(&original, &mut rng);
    let position = relocated.skittles()[0].position();
    assert_ne!(position, Position::new(0, 3));
    let hit = throw_stick(&Stick::new(position), relocated.skittles(), &mut rng);
    let next = relocate_skittles(&relocated, &mut rng);
    assert_ne!(next.skittles(), relocated.skittles());
    assert_eq!(calc_score(&hit).value(), 5);
    assert_eq!(calc_score(&missed).value(), 0);
    assert_eq!(original.skittles()[0].position(), Position::new(0, 3));
}

#[test]
fn 整数の四隅でも隣接する位置へ移動できる() {
    for (x, y) in [
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        let position = Position::new(x, y);
        let original = Field::new(vec![Skittle::new(5).with_position(position)]);
        let mut rng = ConstantRng(u64::MAX);
        let knocked_over = throw_stick(&Stick::new(position), original.skittles(), &mut rng);
        let next = relocate_skittles(&original, &mut rng);
        assert_eq!(calc_score(&knocked_over).value(), 5);
        let moved = next.skittles()[0].position();
        assert_ne!(moved, position);
        assert!((i64::from(moved.x()) - i64::from(x)).abs() <= 1);
        assert!((i64::from(moved.y()) - i64::from(y)).abs() <= 1);
        assert_eq!(original.skittles()[0].position(), position);
    }
}
