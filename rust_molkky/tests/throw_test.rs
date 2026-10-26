use rand::{SeedableRng, rngs::StdRng};
use rust_molkky::{Field, Position, Skittle, Stick, fall_probability, throw_stick};

#[test]
fn 棒とピンの位置関係から確率を計算する() {
    let pin = Skittle::new(5).with_position(Position::new(3, 4));
    assert_eq!(
        fall_probability(&Stick::new(Position::new(3, 4)), &pin),
        1.0
    );
    assert!((fall_probability(&Stick::new(Position::new(0, 0)), &pin) - 1.0 / 6.0).abs() < 1e-12);
    assert!(
        fall_probability(&Stick::new(Position::new(-3, -4)), &pin)
            < fall_probability(&Stick::new(Position::new(0, 0)), &pin)
    );
}

mod support;
use support::ConstantRng;

#[test]
fn 投てきは元の配置を変えず倒れたピンを返す() {
    let position = Position::new(3, 4);
    let stick = Stick::new(position);
    let targets = [Skittle::new(5).with_position(position)];
    let before = targets;
    let knocked_over = throw_stick(&stick, &targets, &mut StdRng::seed_from_u64(42));
    assert_eq!(stick.position(), position);
    assert_eq!(targets, before);
    assert_eq!(knocked_over, targets);
}

#[test]
fn 同じ配置と棒の位置とシードなら同じピンが倒れる() {
    let field = Field::default();
    let before = field.skittles().to_vec();
    let stick = Stick::new(Position::new(0, 0));
    let mut first = StdRng::seed_from_u64(20261024);
    let mut second = StdRng::seed_from_u64(20261024);
    for _ in 0..100 {
        assert_eq!(
            throw_stick(&stick, field.skittles(), &mut first),
            throw_stick(&stick, field.skittles(), &mut second)
        );
    }
    assert_eq!(field.skittles(), before);
}

#[test]
fn 棒の位置を変えると倒れるピンが変わる() {
    let targets = [
        Skittle::new(5).with_position(Position::new(0, 1)),
        Skittle::new(8).with_position(Position::new(0, 4)),
    ];
    let first = throw_stick(
        &Stick::new(Position::new(0, 0)),
        &targets,
        &mut ConstantRng(1 << 62),
    );
    let second = throw_stick(
        &Stick::new(Position::new(0, 4)),
        &targets,
        &mut ConstantRng(1 << 62),
    );
    assert_eq!(first, [targets[0]]);
    assert_eq!(second, [targets[1]]);
}

#[test]
fn 対象が空なら何も倒れない() {
    assert!(throw_stick(&Stick::new(Position::new(0, 0)), &[], &mut ConstantRng(0)).is_empty());
}

#[test]
#[should_panic(expected = "スキットル番号が重複しています: 5")]
fn 投てき対象の同じ番号は重複として拒否する() {
    throw_stick(
        &Stick::new(Position::new(0, 0)),
        &[Skittle::new(5), Skittle::new(5)],
        &mut ConstantRng(0),
    );
}

#[test]
#[should_panic(expected = "スキットル番号が重複しています: 5")]
fn 投てき対象の同じ番号は位置が違っても拒否する() {
    throw_stick(
        &Stick::new(Position::new(0, 0)),
        &[
            Skittle::new(5),
            Skittle::new(5).with_position(Position::new(3, 4)),
        ],
        &mut ConstantRng(0),
    );
}
