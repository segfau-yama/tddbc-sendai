use rust_molkky::{Field, Skittle, ThrowOutcome, ThrowStrategy, calc_score, throw_stick};

#[derive(Clone)]
struct CoordinateField {
    pins: Vec<Skittle>,
    positions: Vec<(f64, f64)>,
}
impl Field for CoordinateField {
    fn skittles(&self) -> &[Skittle] {
        &self.pins
    }
}

// 物理エンジンの代用品。座標更新を返す契約だけを検証し、物理計算は実装しない。
struct MoveFirst;
impl ThrowStrategy<CoordinateField> for MoveFirst {
    type Error = &'static str;
    fn throw(
        &mut self,
        field: &CoordinateField,
    ) -> Result<ThrowOutcome<CoordinateField>, Self::Error> {
        if field.pins.is_empty() {
            return Err("engine failed");
        }
        let mut updated = field.clone();
        updated.positions[0].0 += 1.0;
        Ok(ThrowOutcome::new(updated, vec![field.pins[0]]))
    }
}

#[test]
fn 座標付きfieldとstrategyを動的に渡し新しい状態と倒れた個体を返す() {
    let field = CoordinateField {
        pins: vec![Skittle::new(12), Skittle::new(12)],
        positions: vec![(0.0, 0.0), (2.0, 0.0)],
    };
    let strategy: &mut dyn ThrowStrategy<CoordinateField, Error = &'static str> = &mut MoveFirst;
    let outcome = throw_stick(&field, strategy).unwrap();
    assert_eq!(outcome.field().positions, [(1.0, 0.0), (2.0, 0.0)]);
    assert_eq!(outcome.field().pins, field.pins);
    assert_eq!(outcome.knocked_over(), [field.pins[0]]);
    assert_eq!(calc_score(outcome.knocked_over()).value(), 12);
    assert_eq!(field.positions, [(0.0, 0.0), (2.0, 0.0)]);
}
#[test]
fn エンジンの失敗は零点と区別して返す() {
    let field = CoordinateField {
        pins: vec![],
        positions: vec![],
    };
    assert!(matches!(
        throw_stick(&field, &mut MoveFirst),
        Err("engine failed")
    ));
    assert!(field.pins.is_empty());
}

struct InvalidResult {
    field_pins: Vec<Skittle>,
    knocked: Vec<Skittle>,
}
impl ThrowStrategy<CoordinateField> for InvalidResult {
    type Error = std::convert::Infallible;
    fn throw(
        &mut self,
        field: &CoordinateField,
    ) -> Result<ThrowOutcome<CoordinateField>, Self::Error> {
        let mut updated = field.clone();
        updated.pins = self.field_pins.clone();
        Ok(ThrowOutcome::new(updated, self.knocked.clone()))
    }
}
#[test]
fn 対象外個体と二重カウントと個体集合の変更を拒否する() {
    let pin = Skittle::new(12);
    let field = CoordinateField {
        pins: vec![pin],
        positions: vec![(0.0, 0.0)],
    };
    for (field_pins, knocked) in [
        (vec![pin], vec![Skittle::new(12)]),
        (vec![pin], vec![pin, pin]),
        (vec![Skittle::new(12)], vec![]),
        (vec![pin, pin], vec![]),
        (vec![], vec![]),
    ] {
        let mut strategy = InvalidResult {
            field_pins,
            knocked,
        };
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| throw_stick(
                &field,
                &mut strategy
            )))
            .is_err()
        );
        assert_eq!(field.pins, [pin]);
    }
}
#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 独自fieldでも入力の同一個体の重複を拒否する() {
    let pin = Skittle::new(5);
    let field = CoordinateField {
        pins: vec![pin, pin],
        positions: vec![(0.0, 0.0); 2],
    };
    let _ = throw_stick(&field, &mut MoveFirst);
}
