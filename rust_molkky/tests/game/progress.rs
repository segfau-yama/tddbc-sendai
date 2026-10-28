use super::*;
use rust_molkky::{Name, ScoreBoard, SimpleMolkky, Skittle};

fn team(name: &str, points: &[i32]) -> Team {
    let history = points
        .iter()
        .map(|&n| {
            if n == 0 {
                SimpleMolkky.calc_score(&[])
            } else {
                SimpleMolkky.calc_score(&[Skittle::new(n)])
            }
        })
        .collect();
    Team::new(Name::new(name).unwrap()).with_score_board(ScoreBoard::new(history))
}

#[test]
fn 最後の生存者の勝利は呼び出し側で判定し得点状態は変えない() {
    let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[0; 3])];
    assert!(SimpleMolkky.judge_game(&teams).is_none());
    let winner = winning_team(&SimpleMolkky, &teams).unwrap();
    assert!(std::ptr::eq(winner, &teams[1]));
    assert_eq!(winner.score_board().total(), 0);
    assert_eq!(winner.score_board().state(), ScoreState::Calculated);
}

#[test]
fn 単独参加や複数生存や全失格では生存だけで勝者にしない() {
    for teams in [
        vec![team("赤", &[])],
        vec![team("赤", &[]), team("青", &[])],
        vec![team("赤", &[0; 3]), team("青", &[0; 3])],
    ] {
        assert!(winning_team(&SimpleMolkky, &teams).is_none());
    }
}

#[test]
fn 五十点到達を優先し単独参加でも勝利する() {
    let teams = [team("赤", &[10; 5])];
    assert_eq!(
        winning_team(&SimpleMolkky, &teams).unwrap().id(),
        teams[0].id()
    );
}
