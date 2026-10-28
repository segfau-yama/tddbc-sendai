use rust_molkky::{Molkky, SimpleMolkky, Skittle};

#[test]
fn 番号1を保持する() {
    assert_eq!(Skittle::new(1).number(), 1);
}

#[test]
fn 番号12を保持する() {
    assert_eq!(Skittle::new(12).number(), 12);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 番号0のスキットルは生成時に拒否する() {
    Skittle::new(0);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 番号13のスキットルは生成時に拒否する() {
    Skittle::new(13);
}

#[test]
#[should_panic(expected = "スキットル番号は1〜12です")]
fn 複数本に範囲外の番号が含まれると生成時に拒否する() {
    SimpleMolkky.calc_score(&[Skittle::new(1), Skittle::new(13)]);
}

#[test]
fn 同じ番号でも別個体は別idでコピーはidを保つ() {
    let first = Skittle::new(12);
    let second = Skittle::new(12);
    assert_ne!(first.id(), second.id());
    let copied = first;
    assert_eq!(copied.id(), first.id());
    assert_eq!(SimpleMolkky.calc_score(&[first, second]).value(), 2);
}

#[test]
fn 状態を変えても元の状態と個体識別子と番号を保つ() {
    use rust_molkky::SkittleState;
    let original = Skittle::new(12);
    let updated = original.with_state(SkittleState::Fall);
    assert_eq!(original.state(), SkittleState::Stand);
    assert_eq!(updated.state(), SkittleState::Fall);
    assert_eq!(updated.id(), original.id());
    assert_eq!(updated.number(), original.number());
}
