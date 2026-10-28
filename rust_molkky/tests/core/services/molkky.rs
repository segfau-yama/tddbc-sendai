use rust_molkky::Skittles;
use rust_molkky::{Molkky, Name, ScoreBoard, SimpleMolkky, Skittle, Team};

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
fn 空の実装を使い使用側で得点表を更新できる() {
    struct Rules;
    impl Molkky for Rules {}
    let original = team("赤", &[]).with_turn(true);
    let updated = Rules.hit(
        &rust_molkky::Stick::new(),
        &[Skittle::new(12)],
        &mut |_, pins| pins.to_vec(),
    );
    let scored = original.with_score_board(
        original
            .score_board()
            .add(Skittles::new(&updated).calc_score()),
    );
    assert_eq!(original.score_board().total(), 0);
    assert_eq!(scored.score_board().total(), 12);
    assert_eq!(scored.id(), original.id());
    assert!(scored.turn());
}

#[test]
fn 判定関数に棒と全個体を一度渡し更新後も全個体を返す() {
    use rust_molkky::{JudgeStrategy, SkittleState, Stick};
    let stick = Stick::new();
    let pins = [Skittle::new(12), Skittle::new(12)];
    let mut calls = 0;
    let mut judge = |received: &Stick, skittles: &[Skittle]| {
        calls += 1;
        assert_eq!(received.id(), stick.id());
        assert_eq!(skittles, pins);
        vec![skittles[0].with_state(SkittleState::Fall), skittles[1]]
    };
    let strategy: JudgeStrategy<'_> = &mut judge;
    let rules: &dyn Molkky = &SimpleMolkky;
    let updated = rules.hit(&stick, &pins, strategy);
    assert_eq!(calls, 1);
    assert_eq!(updated.len(), 2);
    assert_eq!(updated[0].id(), pins[0].id());
    assert_eq!(updated[0].state(), SkittleState::Fall);
    assert_eq!(updated[1], pins[1]);
    assert!(pins.iter().all(|pin| pin.state() == SkittleState::Stand));
}

#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 重複した入力は判定関数を呼ぶ前に拒否する() {
    let pin = Skittle::new(5);
    SimpleMolkky.hit(&rust_molkky::Stick::new(), &[pin, pin], &mut |_, _| {
        panic!("呼び出されない")
    });
}

#[test]
fn 判定後の重複と個体の追加と欠落と置換を拒否する() {
    use rust_molkky::Stick;
    let pins = [Skittle::new(5), Skittle::new(5)];
    for invalid in [
        vec![pins[0], pins[0]],
        vec![pins[0]],
        vec![pins[0], pins[1], Skittle::new(5)],
        vec![pins[0], Skittle::new(5)],
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, _| invalid.clone());
            })
            .is_err()
        );
    }
}

#[test]
fn 判定は空の集合と並び替えられた同じ個体集合を返せる() {
    use rust_molkky::Stick;
    assert!(
        SimpleMolkky
            .hit(&Stick::new(), &[], &mut |_, pins| pins.to_vec())
            .is_empty()
    );
    let pins = [Skittle::new(5), Skittle::new(8)];
    let updated = SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, pins| {
        pins.iter().rev().copied().collect()
    });
    assert_eq!(updated, [pins[1], pins[0]]);
}
