use rust_molkky::{GameStatus, ScoreBoard, Skittle, Team, calc_score, judge_game};

fn team(name: &str, points: &[i32]) -> Team {
    let board = points.iter().fold(ScoreBoard::new(), |board, &number| {
        let pins = if number == 0 {
            vec![]
        } else {
            vec![Skittle::new(number)]
        };
        board.add(calc_score(&pins))
    });
    Team::new(name).with_score_board(board)
}

#[test]
fn 五十点に到達したチームを勝者とする() {
    assert_eq!(
        judge_game(&[team("赤", &[]), team("青", &[10; 5])]),
        GameStatus::Won { team_index: 1 }
    );
}

#[test]
fn 元の参加者が複数なら最後の生存チームが零点でも勝つ() {
    let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[0; 3])];
    assert_eq!(judge_game(&teams), GameStatus::Won { team_index: 1 });
    assert_eq!(teams[1].score_board().value(), 0);
}

#[test]
fn 単独参加は五十点まで進行し失格なら勝者なしで終わる() {
    assert_eq!(judge_game(&[team("赤", &[])]), GameStatus::InProgress);
    assert_eq!(
        judge_game(&[team("赤", &[0; 3])]),
        GameStatus::FinishedWithoutWinner
    );
    assert_eq!(
        judge_game(&[team("赤", &[10; 5])]),
        GameStatus::Won { team_index: 0 }
    );
}

#[test]
fn 複数チームが残る間は進行し全員失格なら勝者なしになる() {
    assert_eq!(
        judge_game(&[team("赤", &[0; 3]), team("青", &[12; 4]), team("緑", &[])]),
        GameStatus::InProgress
    );
    assert_eq!(
        judge_game(&[team("赤", &[0; 3]), team("青", &[0; 3])]),
        GameStatus::FinishedWithoutWinner
    );
    assert_eq!(
        judge_game(&[team("赤", &[12, 12, 12, 12, 3]), team("青", &[])]),
        GameStatus::InProgress
    );
}

#[test]
#[should_panic(expected = "チームは1件以上必要です")]
fn 空の入力を拒否する() {
    judge_game(&[]);
}

#[test]
#[should_panic(expected = "勝者が複数います")]
fn 複数の五十点到達チームを拒否する() {
    judge_game(&[team("赤", &[10; 5]), team("青", &[10; 5])]);
}

#[test]
fn 終了後の不正な履歴を独立した判定関数でも拒否する() {
    for points in [&[0, 0, 0, 1][..], &[10, 10, 10, 10, 10, 1][..]] {
        assert!(std::panic::catch_unwind(|| judge_game(&[team("赤", points)])).is_err());
    }
}
