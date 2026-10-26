use super::ScoreBoardTui;
use rust_molkky::core::{ScoreBoard, Skittle, calc_score};

#[test]
fn 履歴は0点を含め直近六投を表示する() {
    let mut board = ScoreBoard::new();
    for number in [1, 2, 0, 3, 4, 5, 0, 6] {
        let point = if number == 0 {
            calc_score(&[])
        } else {
            calc_score(&[Skittle::new(number)])
        };
        board = board.add(point);
    }
    let view = ScoreBoardTui::new(&board);
    assert_eq!(view.value_label(), "21 / 50");
    assert_eq!(view.throw_count_label(), "8");
    assert_eq!(view.recent_points_label(), "0 3 4 5 0 6");
    assert!(view.change_notice().is_empty());
}

#[test]
fn 補正と失格の説明は採点側の状態に従う() {
    let twelve = calc_score(&[Skittle::new(12)]);
    let mut board = ScoreBoard::new();
    for _ in 0..4 {
        board = board.add(twelve);
    }
    board = board.add(calc_score(&[Skittle::new(1)]));
    let reset = board.add(calc_score(&[Skittle::new(2)]));
    let view = ScoreBoardTui::new(&reset);
    assert_eq!(view.value_label(), "25 / 50");
    assert!(view.change_notice().contains("50点を超過したため25点へ"));
    assert!(
        ScoreBoardTui::new(&reset.add(twelve))
            .change_notice()
            .is_empty()
    );
    let missed = calc_score(&[]);
    let disqualified = ScoreBoard::new().add(missed).add(missed).add(missed);
    assert!(
        ScoreBoardTui::new(&disqualified)
            .change_notice()
            .contains("3回連続0点のため失格")
    );
}
