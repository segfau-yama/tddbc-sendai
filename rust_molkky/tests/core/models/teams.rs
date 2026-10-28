use rust_molkky::Skittles;
use rust_molkky::{Name, ScoreBoard, ScoreState, Skittle, Team, Teams};

fn team(name: &str, points: &[i32]) -> Team {
    let history = points
        .iter()
        .map(|&n| {
            if n == 0 {
                Skittles::new(&[]).calc_score()
            } else {
                Skittles::new(&[Skittle::new(n)]).calc_score()
            }
        })
        .collect();
    Team::new(Name::new(name).unwrap()).with_score_board(ScoreBoard::new(history))
}

#[test]
fn 次の手番へ循環し失格を飛ばして元のチームを変更しない() {
    let teams = [
        team("赤", &[]).with_turn(true),
        team("青", &[0; 3]),
        team("緑", &[5]),
    ];
    let next = Teams::new(&teams).next_team();
    assert_eq!(
        next.iter().map(Team::turn).collect::<Vec<_>>(),
        [false, false, true]
    );
    assert_eq!(
        teams.iter().map(Team::turn).collect::<Vec<_>>(),
        [true, false, false]
    );
    for (before, after) in teams.iter().zip(&next) {
        assert_eq!(before.id(), after.id());
        assert_eq!(before.name(), after.name());
        assert_eq!(
            before.score_board().history(),
            after.score_board().history()
        );
        assert_eq!(before.score_board().state(), after.score_board().state());
    }
    let wrapped = Teams::new(&next).next_team();
    assert_eq!(
        wrapped.iter().map(Team::turn).collect::<Vec<_>>(),
        [true, false, false]
    );
}

#[test]
fn 手番未設定なら先頭の有効チームで開始し単独参加なら同じチームへ戻る() {
    let teams = [team("赤", &[0; 3]), team("青", &[]), team("緑", &[])];
    let next = Teams::new(&teams).next_team();
    assert_eq!(
        next.iter().map(Team::turn).collect::<Vec<_>>(),
        [false, true, false]
    );
    let solo = Teams::new(&[team("赤", &[])]).next_team();
    assert!(solo[0].turn());
    assert!(Teams::new(&solo).next_team()[0].turn());
}

#[test]
fn 空と全失格と勝利と最後の生存者では手番を解除する() {
    assert!(Teams::new(&[]).next_team().is_empty());
    for teams in [
        vec![team("赤", &[0; 3]).with_turn(true)],
        vec![team("赤", &[0; 3]), team("青", &[0; 3]).with_turn(true)],
        vec![team("赤", &[10; 5]).with_turn(true), team("青", &[])],
        vec![team("赤", &[0; 3]).with_turn(true), team("青", &[])],
    ] {
        let next = Teams::new(&teams).next_team();
        assert!(next.iter().all(|team| !team.turn()));
        assert_eq!(next.len(), teams.len());
    }
}

#[test]
fn 失格になった現在のチームから次の有効チームへ進める() {
    let teams = [
        team("赤", &[0; 3]).with_turn(true),
        team("青", &[]),
        team("緑", &[]),
    ];
    let next = Teams::new(&teams).next_team();
    assert_eq!(
        next.iter().map(Team::turn).collect::<Vec<_>>(),
        [false, true, false]
    );
}

#[test]
#[should_panic(expected = "手番のチームが複数います")]
fn 複数の手番を拒否する() {
    Teams::new(&[
        team("赤", &[]).with_turn(true),
        team("青", &[]).with_turn(true),
    ])
    .next_team();
}

#[test]
fn 勝利状態のチームそのものを返し手番には依存しない() {
    let teams = [team("赤", &[]).with_turn(true), team("青", &[10; 5])];
    let winner = Teams::new(&teams).judge_game().unwrap();
    assert!(std::ptr::eq(winner, &teams[1]));
    assert_eq!(winner.score_board().state(), ScoreState::Won);
    assert!(!winner.turn());
}

#[test]
fn 勝利状態でなければ最後の生存者や全失格でも勝者を返さない() {
    for teams in [
        vec![team("赤", &[])],
        vec![team("赤", &[0; 3]), team("青", &[])],
        vec![team("赤", &[0; 3])],
    ] {
        assert!(Teams::new(&teams).judge_game().is_none());
    }
}

#[test]
#[should_panic(expected = "勝者が複数います")]
fn 複数の勝利状態を拒否する() {
    Teams::new(&[team("赤", &[10; 5]), team("青", &[10; 5])]).judge_game();
}

#[test]
#[should_panic(expected = "チームIDが重複しています")]
fn 同じチームの二重登録を拒否する() {
    let first = team("赤", &[]);
    Teams::new(&[first.clone(), first]).judge_game();
}

#[test]
#[should_panic(expected = "チームIDが重複しています")]
fn 手番の更新でも同じチームの二重登録を拒否する() {
    let first = team("赤", &[]);
    Teams::new(&[first.clone(), first]).next_team();
}

#[test]
#[should_panic(expected = "チームは1件以上必要です")]
fn 空の参加者で勝敗を判定できない() {
    Teams::new(&[]).judge_game();
}
