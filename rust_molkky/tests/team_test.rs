use rust_molkky::{Skittle, Team, Throw};

#[test]
fn 名称を保持し得点は0から始まる() {
    let team = Team::new("赤チーム");
    assert_eq!(team.name(), "赤チーム");
    assert_eq!(team.score().value(), 0);
    assert!(team.score().history().is_empty());
}

#[test]
fn 投てきの点数を返し得点には一回だけ加算する() {
    let mut team = Team::new("赤チーム");
    let thrown = Throw::new(&[Skittle::new(11)]);
    assert_eq!(team.record_throw(&thrown), 11);
    assert_eq!(team.score().value(), 11);
    assert_eq!(team.score().history(), &[11]);
}

#[test]
fn 名称は空文字も同じ名称もそのまま保持する() {
    assert_eq!(Team::new("").name(), "");
    assert_eq!(Team::new("同じ名前").name(), Team::new("同じ名前").name());
}

#[test]
fn 超過しても返す点数は投てきの点数で得点は25点になる() {
    let mut team = Team::new("赤チーム");
    let twelve = Throw::new(&[Skittle::new(12)]);
    for _ in 0..4 {
        team.record_throw(&twelve);
    }
    assert_eq!(team.record_throw(&Throw::new(&[Skittle::new(3)])), 3);
    assert_eq!(team.score().value(), 25);
    assert_eq!(team.score().history(), &[12, 12, 12, 12, 3]);
}
