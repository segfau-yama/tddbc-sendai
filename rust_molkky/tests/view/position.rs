use super::Position;
#[test]
fn 座標を保持し描画用の表記へ変換する() {
    let p = Position::new(3, -4);
    assert_eq!((p.x(), p.y()), (3, -4));
    assert_eq!(p.canvas(), (3.0, -4.0));
    assert_eq!(p.label(), "(3, -4)");
    assert_eq!(p.table_label(), "(  3, -4)");
}
#[test]
fn 移動は元の座標を保持し整数の境界では拒否する() {
    let p = Position::new(3, 4);
    assert_eq!(p.translated(-1, 2), Some(Position::new(2, 6)));
    assert_eq!(p, Position::new(3, 4));
    for (p, dx, dy) in [
        (Position::new(i32::MAX, 0), 1, 0),
        (Position::new(i32::MIN, 0), -1, 0),
        (Position::new(0, i32::MAX), 0, 1),
        (Position::new(0, i32::MIN), 0, -1),
    ] {
        assert_eq!(p.translated(dx, dy), None);
    }
}
