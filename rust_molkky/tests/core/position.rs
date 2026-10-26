use rust_molkky::Position;

#[test]
fn 指定した座標を保持する() {
    let position = Position::new(3, -4);
    assert_eq!(position.x(), 3);
    assert_eq!(position.y(), -4);
}

#[test]
fn 原点から3と4離れた位置までの距離は5になる() {
    assert_eq!(Position::new(0, 0).distance_to(Position::new(3, 4)), 5.0);
}

#[test]
fn 同じ位置は距離0で負の座標でも距離は対称になる() {
    let a = Position::new(-3, -4);
    let b = Position::new(0, 0);
    assert_eq!(a.distance_to(a), 0.0);
    assert_eq!(a.distance_to(b), 5.0);
    assert_eq!(b.distance_to(a), 5.0);
}

#[test]
fn 整数の端同士でも距離を求められる() {
    let a = Position::new(i32::MIN, 0);
    let b = Position::new(i32::MAX, 0);
    assert_eq!(a.distance_to(b), 4_294_967_295.0);
}
