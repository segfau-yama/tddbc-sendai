# 投てき・採点・得点履歴・ゲーム管理

## 採点して得点表へ記録する

```rust
use rust_molkky::{ScoreBoard, Skittle, calc_score};
let first = calc_score(&[Skittle::new(5)]);
let second = calc_score(&[Skittle::new(11)]);
let mut board = ScoreBoard::new();
board = board.add(first);
board = board.add(second);
assert_eq!(first.value(), 5);
assert_eq!(board.value(), 16);
assert_eq!(board.history(), &[first, second]);
```

## Skittleの番号にNoneは指定できない

```compile_fail
use rust_molkky::Skittle;
Skittle::new(None);
```

## 採点入力とその要素にNoneは指定できない

```compile_fail
use rust_molkky::calc_score;
calc_score(None);
```

```compile_fail
use rust_molkky::{Skittle, calc_score};
calc_score(&[None::<Skittle>]);
```

## 得点表にNoneは追加できない

```compile_fail
use rust_molkky::ScoreBoard;
ScoreBoard::new().add(None);
```

## 無効なPointを直接生成できない

```compile_fail
use rust_molkky::Point;
Point(99);
```

## 50点超過では履歴を残して25点へ戻す

```rust
use rust_molkky::{ScoreBoard, Skittle, calc_score};
let twelve = calc_score(&[Skittle::new(12)]);
let three = calc_score(&[Skittle::new(3)]);
let mut board = ScoreBoard::new();
for _ in 0..4 { board = board.add(twelve); }
board = board.add(three);
assert_eq!(board.value(), 25);
assert_eq!(board.history(), &[twelve, twelve, twelve, twelve, three]);
board = board.add(twelve);
assert_eq!(board.value(), 37);
assert_eq!(board.history(), &[twelve, twelve, twelve, twelve, three, twelve]);
```

## 複数チームで50点ちょうどの勝者を判定する

```rust
use rust_molkky::{Molkky, Skittle, Team, calc_score};
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let ten = calc_score(&[Skittle::new(10)]);
for _ in 0..5 { game = game.record_point(1, ten); }
assert_eq!(game.result(), Some(1));
assert_eq!(game.teams()[0].score_board().value(), 0);
assert_eq!(game.teams()[1].score_board().value(), 50);
```

## 履歴とチーム一覧からゲームの状態を書き換えることはできない

```compile_fail
use rust_molkky::{ScoreBoard, Skittle, calc_score};
let point = calc_score(&[Skittle::new(5)]);
let mut board = ScoreBoard::new();
board = board.add(point);
board.history()[0] = calc_score(&[Skittle::new(12)]);
```

```compile_fail
use rust_molkky::{Molkky, Skittle, Team, calc_score};
let mut game = Molkky::new(vec![Team::new("赤")]);
game.teams()[0] = game.teams()[0].record_point(calc_score(&[Skittle::new(5)]));
```

## 配置を変更せず倒れたピンを返す

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Position, Skittle, Stick, calc_score, throw_stick};
let mut rng = StdRng::seed_from_u64(42);
let position = Position::new(0, 0);
let targets = [Skittle::new(5).with_position(position)];
let knocked_over = throw_stick(&Stick::new(position), &targets, &mut rng);
assert_eq!(knocked_over, targets);
assert_eq!(calc_score(&knocked_over).value(), 5);
assert_eq!(targets[0].position(), position);
```

## 投てき対象にNoneは指定できない

```compile_fail
use rust_molkky::{Position, Stick, throw_stick};
throw_stick(&Stick::new(Position::new(0, 0)), None, &mut rand::rng());
```

## 外側で投てきと不変な状態の更新を組み合わせる

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Molkky, Position, Stick, Team, calc_score, throw_stick};
let mut rng = StdRng::seed_from_u64(20261024);
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let field = Field::default();
let knocked_over = throw_stick(&Stick::new(Position::new(0, 0)), field.skittles(), &mut rng);
let original_positions = field.skittles().to_vec();
let next_field = field.relocate(&mut rng);
assert_eq!(field.skittles(), original_positions);
assert_ne!(next_field.skittles(), field.skittles());
let point = calc_score(&knocked_over);
game = game.record_point(0, point);
assert_eq!(game.teams()[0].score_board().history(), &[point]);
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
use rust_molkky::{Field, Position, Skittle};
let mut field = Field::default();
field.skittles()[0] = Skittle::new(12);
```

## 得点表の更新は元の履歴を変更しない

```rust
use rust_molkky::{ScoreBoard, Skittle, calc_score};
let five = calc_score(&[Skittle::new(5)]);
let eleven = calc_score(&[Skittle::new(11)]);
let original = ScoreBoard::new().add(five);
let updated = original.add(eleven);
assert_eq!(original.history(), &[five]);
assert_eq!(updated.history(), &[five, eleven]);
```

## 更新結果の破棄はunused_must_useで検出できる

```compile_fail
#![deny(unused_must_use)]
use rust_molkky::{ScoreBoard, calc_score};
let board = ScoreBoard::new();
board.add(calc_score(&[]));
```
