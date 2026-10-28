use rust_molkky::Name;
use rust_molkky::{Molkky, SimpleMolkky, Skittle, Team};

#[test]
fn 名称を保持し得点は0から始まる() {
    let team = Team::new(Name::new("赤チーム").unwrap());
    assert_eq!(team.name().as_str(), "赤チーム");
    assert_eq!(team.score_board().total(), 0);
    assert!(team.score_board().history().is_empty());
}

#[test]
fn 得点表を指定して元のチームを変えずに再構成する() {
    let mut team = Team::new(Name::new("赤チーム").unwrap());
    let thrown = SimpleMolkky.calc_score(&[Skittle::new(11)]);
    assert_eq!(thrown.value(), 11);
    team = team.with_score_board(team.score_board().add(thrown));
    assert_eq!(team.score_board().total(), 11);
    assert_eq!(
        team.score_board()
            .history()
            .iter()
            .map(|point| point.value())
            .collect::<Vec<_>>(),
        &[11]
    );
}

#[test]
fn 同名でも異なるチームを作成できる() {
    assert_eq!(
        Team::new(Name::new("同じ名前").unwrap()).name().as_str(),
        Team::new(Name::new("同じ名前").unwrap()).name().as_str()
    );
}

#[test]
fn 超過しても生の点数を保持して得点は25点になる() {
    let mut team = Team::new(Name::new("赤チーム").unwrap());
    let twelve = SimpleMolkky.calc_score(&[Skittle::new(12)]);
    for _ in 0..4 {
        team = team.with_score_board(team.score_board().add(twelve));
    }
    let point = SimpleMolkky.calc_score(&[Skittle::new(3)]);
    assert_eq!(point.value(), 3);
    team = team.with_score_board(team.score_board().add(point));
    assert_eq!(team.score_board().total(), 25);
    assert_eq!(
        team.score_board()
            .history()
            .iter()
            .map(|point| point.value())
            .collect::<Vec<_>>(),
        &[12, 12, 12, 12, 3]
    );
}

mod immutability {
    use super::*;

    #[test]
    fn チームへの記録でも元のチームは変わらない() {
        let original = Team::new(Name::new("赤").unwrap());
        let updated = original.with_score_board(
            original
                .score_board()
                .add(SimpleMolkky.calc_score(&[Skittle::new(11)])),
        );
        assert_eq!(original.score_board().total(), 0);
        assert!(original.score_board().history().is_empty());
        assert_eq!(updated.score_board().total(), 11);
        assert_eq!(updated.name().as_str(), original.name().as_str());
    }
}

#[test]
fn 同名でも別チームのidは異なりcloneと得点更新ではidを維持する() {
    let first = Team::new(Name::new("同じ名前").unwrap());
    let second = Team::new(Name::new("同じ名前").unwrap());
    assert_ne!(first.id(), second.id());
    assert_eq!(first.clone().id(), first.id());
    let updated = first.with_score_board(
        first
            .score_board()
            .add(SimpleMolkky.calc_score(&[Skittle::new(5)])),
    );
    assert_eq!(updated.id(), first.id());
    let mut teams = [first, second];
    let id = teams[0].id();
    teams.swap(0, 1);
    assert_eq!(
        teams
            .iter()
            .find(|team| team.id() == id)
            .unwrap()
            .name()
            .as_str(),
        "同じ名前"
    );
}

#[test]
fn 手番の更新でも元のチームとidを保ち得点更新でも手番を維持する() {
    let original = Team::new(Name::new("赤").unwrap());
    assert!(!original.turn());
    let active = original.with_turn(true);
    assert!(active.turn());
    assert!(!original.turn());
    assert_eq!(active.id(), original.id());
    let scored = active.with_score_board(
        active
            .score_board()
            .add(SimpleMolkky.calc_score(&[Skittle::new(5)])),
    );
    assert!(scored.turn());
    assert_eq!(scored.clone().id(), active.id());
    assert_eq!(scored.score_board().total(), 5);
}
