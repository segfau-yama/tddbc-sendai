use rust_molkky::{Skittle, Team, calc_score};

#[test]
fn 名称を保持し得点は0から始まる() {
    let team = Team::new("赤チーム");
    assert_eq!(team.name(), "赤チーム");
    assert_eq!(team.score_board().value(), 0);
    assert!(team.score_board().history().is_empty());
}

#[test]
fn 得点表を指定して元のチームを変えずに再構成する() {
    let mut team = Team::new("赤チーム");
    let thrown = calc_score(&[Skittle::new(11)]);
    assert_eq!(thrown.value(), 11);
    team = team.with_score_board(team.score_board().add(thrown));
    assert_eq!(team.score_board().value(), 11);
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
fn 名称は空文字も同じ名称もそのまま保持する() {
    assert_eq!(Team::new("").name(), "");
    assert_eq!(Team::new("同じ名前").name(), Team::new("同じ名前").name());
}

#[test]
fn 超過しても生の点数を保持して得点は25点になる() {
    let mut team = Team::new("赤チーム");
    let twelve = calc_score(&[Skittle::new(12)]);
    for _ in 0..4 {
        team = team.with_score_board(team.score_board().add(twelve));
    }
    let point = calc_score(&[Skittle::new(3)]);
    assert_eq!(point.value(), 3);
    team = team.with_score_board(team.score_board().add(point));
    assert_eq!(team.score_board().value(), 25);
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
        let original = Team::new("赤");
        let updated =
            original.with_score_board(original.score_board().add(calc_score(&[Skittle::new(11)])));
        assert_eq!(original.score_board().value(), 0);
        assert!(original.score_board().history().is_empty());
        assert_eq!(updated.score_board().value(), 11);
        assert_eq!(updated.name(), original.name());
    }
}
