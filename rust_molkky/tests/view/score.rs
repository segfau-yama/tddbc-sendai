use super::ScoreBoardTui;
#[test]
fn 得点と投数と直近六投を表示値から作る() {
    let board = ScoreBoardTui {
        value: 21,
        target: 50,
        history: vec![1, 2, 0, 3, 4, 5, 0, 6],
        notice: String::new(),
    };
    assert_eq!(board.value_label(), "21 / 50");
    assert_eq!(board.throw_count_label(), "8");
    assert_eq!(board.recent_points_label(), "0 3 4 5 0 6");
}
