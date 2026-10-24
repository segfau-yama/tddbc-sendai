# モルックの点数・履歴・ゲーム管理

## 使用例

```rust
use rust_molkky::{Score, Skittle, Throw};

let first = Throw::new(&[Skittle::new(5)]);
let second = Throw::new(&[Skittle::new(11)]);
assert_eq!(Score::new(&[&first, &second]).value(), 16);
```

## Skittleの番号にNoneは指定できない

```compile_fail
use rust_molkky::Skittle;
Skittle::new(None);
```

## Throwのスライスと要素にNoneは指定できない

```compile_fail
use rust_molkky::Throw;
Throw::new(None);
```

```compile_fail
use rust_molkky::{Skittle, Throw};
Throw::new(&[None::<Skittle>]);
```

## Scoreのスライスと要素にNoneは指定できない

```compile_fail
use rust_molkky::Score;
Score::new(None);
```

```compile_fail
use rust_molkky::{Score, Throw};
Score::new(&[None::<&Throw>]);
```

## 50点超過では履歴を残して25点へ戻す

```rust
use rust_molkky::{Score, Skittle, Throw};
let twelve = Throw::new(&[Skittle::new(12)]);
let three = Throw::new(&[Skittle::new(3)]);
let score = Score::new(&[&twelve, &twelve, &twelve, &twelve, &three]);
assert_eq!(score.value(), 25);
assert_eq!(score.history(), &[12, 12, 12, 12, 3]);
let next = score.with_throw(&twelve);
assert_eq!(score.value(), 25);
assert_eq!(next.value(), 37);
```

## 複数チームで50点ちょうどの勝者を判定する

```rust
use rust_molkky::{Molkky, Skittle, Team, Throw};
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let ten = Throw::new(&[Skittle::new(10)]);
for _ in 0..5 { game.record_throw(1, &ten); }
assert_eq!(game.result(), Some(1));
assert_eq!(game.teams()[0].score().value(), 0);
assert_eq!(game.teams()[1].score().value(), 50);
```

## 履歴とチーム一覧からゲームの状態を書き換えることはできない

```compile_fail
use rust_molkky::{Score, Skittle, Throw};
let thrown = Throw::new(&[Skittle::new(5)]);
let mut score = Score::new(&[&thrown]);
score.history()[0] = 12;
```

```compile_fail
use rust_molkky::{Molkky, Skittle, Team, Throw};
let mut game = Molkky::new(vec![Team::new("赤")]);
game.teams()[0].record_throw(&Throw::new(&[Skittle::new(5)]));
```

## 外側でランダムな投てきを生成する

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{random_throw, Field, Molkky, Position, Team};
let mut rng = StdRng::seed_from_u64(20261024);
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let mut field = Field::default();
let thrown = random_throw(&mut field, Position::new(0, 0), &mut rng);
let points = game.record_throw(0, &thrown);
assert_eq!(game.teams()[0].score().history(), &[points]);
```

## 座標を指定して配置し距離から倒れる確率を求める

```rust
use rust_molkky::{Field, Position, Skittle};
let pin = Skittle::new(5).with_position(Position::new(3, 4));
assert_eq!(pin.position().distance_to(Position::new(0, 0)), 5.0);
assert_eq!(pin.fall_probability(Position::new(3, 4)), 1.0);
assert!(pin.fall_probability(Position::new(-3, -4)) < pin.fall_probability(Position::new(0, 0)));
let field = Field::new(vec![pin]);
assert_eq!(field.skittles()[0].position(), Position::new(3, 4));
```

## Fieldの配置を読み取りAPIから書き換えることはできない

```compile_fail
use rust_molkky::{Field, Skittle};
let mut field = Field::default();
field.skittles()[0] = Skittle::new(12);
```
