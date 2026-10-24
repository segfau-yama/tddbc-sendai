use rand::{RngCore, SeedableRng, rngs::StdRng};
use rust_molkky::{Field, Position, Skittle, random_throw};

struct ConstantRng(u64);

impl RngCore for ConstantRng {
    fn next_u32(&mut self) -> u32 {
        self.0 as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            chunk.copy_from_slice(&self.0.to_le_bytes()[..chunk.len()]);
        }
    }
}

#[test]
fn すべてのピンが倒れる乱数なら12点になる() {
    assert_eq!(
        random_throw(
            &mut Field::default(),
            Position::new(0, 0),
            &mut ConstantRng(0)
        )
        .calc_point(),
        12
    );
}

#[test]
fn どのピンも倒れない乱数なら0点になる() {
    assert_eq!(
        random_throw(
            &mut Field::default(),
            Position::new(0, 0),
            &mut ConstantRng(u64::MAX)
        )
        .calc_point(),
        0
    );
}

#[test]
fn 同じシードなら同じ投てきの点数列を再現できる() {
    let mut first = StdRng::seed_from_u64(20261024);
    let mut second = StdRng::seed_from_u64(20261024);
    let mut first_field = Field::default();
    let mut second_field = Field::default();
    for _ in 0..100 {
        let points = random_throw(&mut first_field, Position::new(0, 0), &mut first).calc_point();
        assert!((0..=12).contains(&points));
        assert_eq!(
            points,
            random_throw(&mut second_field, Position::new(0, 0), &mut second).calc_point()
        );
        assert_eq!(first_field.skittles(), second_field.skittles());
    }
}

#[test]
fn 同じ乱数でも近いスキットルだけが倒れる() {
    let mut field = Field::new(vec![
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(0, 4)),
    ]);
    let thrown = random_throw(&mut field, Position::new(0, 0), &mut ConstantRng(1 << 62));
    assert_eq!(thrown.calc_point(), 5);
}

#[test]
fn 投てき場所を変えると倒れるスキットルが変わる() {
    let pins = vec![
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(0, 4)),
    ];
    let near_five = random_throw(
        &mut Field::new(pins.clone()),
        Position::new(0, 0),
        &mut ConstantRng(1 << 62),
    );
    let near_eight = random_throw(
        &mut Field::new(pins),
        Position::new(0, 4),
        &mut ConstantRng(1 << 62),
    );
    assert_eq!(near_five.calc_point(), 5);
    assert_eq!(near_eight.calc_point(), 8);
}

#[test]
fn 投てきのたびに全ピンの位置が変わり番号と本数と点数は保たれる() {
    let mut field = Field::default();
    let numbers: Vec<_> = field.skittles().iter().map(Skittle::number).collect();
    for _ in 0..3 {
        let before = field.skittles().to_vec();
        let thrown = random_throw(&mut field, Position::new(0, 0), &mut ConstantRng(0));
        assert_eq!(thrown.calc_point(), 12);
        assert_eq!(field.skittles().len(), 12);
        assert_eq!(
            field
                .skittles()
                .iter()
                .map(Skittle::number)
                .collect::<Vec<_>>(),
            numbers
        );
        for (old, new) in before.iter().zip(field.skittles()) {
            assert_ne!(old.position(), new.position());
            assert!((i64::from(old.position().x()) - i64::from(new.position().x())).abs() <= 1);
            assert!((i64::from(old.position().y()) - i64::from(new.position().y())).abs() <= 1);
        }
    }
}

#[test]
fn 空のフィールドは何も移動せず0点になる() {
    let mut field = Field::new(vec![]);
    assert_eq!(
        random_throw(&mut field, Position::new(0, 0), &mut ConstantRng(0)).calc_point(),
        0
    );
    assert!(field.skittles().is_empty());
}

#[test]
fn 倒れなかったピンも移動し移動前の投てきの点数は変わらない() {
    let mut field = Field::new(vec![Skittle::new(5).with_position(Position::new(0, 3))]);
    let thrown = random_throw(&mut field, Position::new(0, 0), &mut ConstantRng(u64::MAX));
    assert_eq!(thrown.calc_point(), 0);
    let before = field.skittles()[0].position();
    assert_ne!(before, Position::new(0, 3));
    let hit = random_throw(&mut field, before, &mut ConstantRng(u64::MAX));
    assert_eq!(hit.calc_point(), 5);
    random_throw(&mut field, Position::new(0, 0), &mut ConstantRng(0));
    assert_eq!(hit.calc_point(), 5);
    assert_eq!(thrown.calc_point(), 0);
}

#[test]
fn 整数の四隅でも隣接する位置へ移動できる() {
    for (x, y) in [
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        let original = Position::new(x, y);
        let mut field = Field::new(vec![Skittle::new(5).with_position(original)]);
        let thrown = random_throw(&mut field, original, &mut ConstantRng(u64::MAX));
        assert_eq!(thrown.calc_point(), 5);
        let moved = field.skittles()[0].position();
        assert_ne!(moved, original);
        assert!((i64::from(moved.x()) - i64::from(x)).abs() <= 1);
        assert!((i64::from(moved.y()) - i64::from(y)).abs() <= 1);
    }
}
