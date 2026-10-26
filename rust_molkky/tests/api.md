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
assert!(board.was_reset());
assert!(!board.is_winner());
assert_eq!(board.history(), &[twelve, twelve, twelve, twelve, three]);
board = board.add(twelve);
assert_eq!(board.value(), 37);
assert!(!board.was_reset());
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
game.teams()[0] = game.teams()[0].with_score_board(
    game.teams()[0].score_board().add(calc_score(&[Skittle::new(5)]))
);
```

## 入力を変更せず倒れたピンを返す

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Skittle, Stick, calc_score, throw_stick};
let mut rng = StdRng::seed_from_u64(42);
let targets = [Skittle::new(5)];
let knocked_over = throw_stick(&Stick::new(1.0), &targets, &mut rng);
assert_eq!(knocked_over, targets);
assert_eq!(calc_score(&knocked_over).value(), 5);
assert_eq!(targets, [Skittle::new(5)]);
```

## 投てき対象にNoneは指定できない

```compile_fail
use rust_molkky::{Stick, throw_stick};
throw_stick(&Stick::default(), None, &mut rand::rng());
```

## 外側で投てきと不変な状態の更新を組み合わせる

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Molkky, Skittle, Stick, Team, calc_score, throw_stick};
let original = Molkky::new(vec![Team::new("赤"), Team::new("青")])
    .with_field(Field::new(vec![Skittle::new(5)]))
    .with_stick(Stick::new(1.0));
let mut rng = StdRng::seed_from_u64(42);
let knocked_over = throw_stick(original.stick(), original.field().skittles(), &mut rng);
let updated = original.record_point(0, calc_score(&knocked_over));
assert_eq!(original.teams()[0].score_board().value(), 0);
assert_eq!(updated.teams()[0].score_board().value(), 5);
assert_eq!(updated.field().skittles(), original.field().skittles());
```

## 座標なしに棒ごとの確率を指定する

```rust
use rust_molkky::{Skittle, Stick, throw_stick};
let pins = [Skittle::new(5), Skittle::new(8)];
assert_eq!(Stick::default().fall_probability(), 0.25);
assert!(throw_stick(&Stick::new(0.0), &pins, &mut rand::rng()).is_empty());
assert_eq!(throw_stick(&Stick::new(1.0), &pins, &mut rand::rng()), pins);
```

## Fieldのピン集合を読み取りAPIから書き換えることはできない

```compile_fail
use rust_molkky::{Field, Skittle};
let field = Field::default();
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

## チームは得点表を差し替えて新しい値を返す

```rust
use rust_molkky::{ScoreBoard, Skittle, Team, calc_score};
let original = Team::new("赤");
let point = calc_score(&[Skittle::new(5)]);
let updated = original.with_score_board(ScoreBoard::new().add(point));
assert_eq!(original.score_board().value(), 0);
assert_eq!(updated.score_board().value(), 5);
assert_eq!(original.name(), updated.name());
```

## 3回連続0点で失格にし二チーム以上残っていれば続ける

```rust
use rust_molkky::{Molkky, ScoreBoard, Skittle, Team, calc_score};
let missed = calc_score(&[]);
let disqualified = ScoreBoard::new().add(missed).add(missed).add(missed);
assert!(disqualified.is_disqualified());
let mut game = Molkky::new(vec![
    Team::new("赤").with_score_board(disqualified),
    Team::new("青"),
    Team::new("緑"),
]);
assert!(!game.is_finished());
assert_eq!(game.result(), None);
game = game.record_point(1, calc_score(&[Skittle::new(5)]));
assert_eq!(game.teams()[1].score_board().value(), 5);
for _ in 0..3 { game = game.record_point(1, missed); }
assert!(game.is_finished());
assert_eq!(game.result(), Some(2)); // 最後まで失格していない緑チームの勝利。
assert_eq!(game.teams()[2].score_board().value(), 0);
```

## FieldとStickはゲーム共通で不変に差し替える

```rust
use rust_molkky::{Field, Molkky, Skittle, Stick, Team, calc_score};
let original = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let configured = original.with_field(Field::new(vec![Skittle::new(5)]))
    .with_stick(Stick::new(0.75));
let recorded = configured.record_point(1, calc_score(&[Skittle::new(5)]));
assert_eq!(original.field().skittles().len(), 12);
assert_eq!(original.stick().fall_probability(), 0.25);
assert_eq!(recorded.field().skittles(), configured.field().skittles());
assert_eq!(*recorded.stick(), *configured.stick());
assert_eq!(recorded.teams()[1].score_board().value(), 5);
```

```compile_fail
use rust_molkky::{Field, Molkky, Team};
let game = Molkky::new(vec![Team::new("赤")]);
*game.field() = Field::new(vec![]);
```

```compile_fail
use rust_molkky::{Molkky, Stick, Team};
let game = Molkky::new(vec![Team::new("赤")]);
*game.stick() = Stick::new(0.5);
```

## coreモジュールからも同じ公開APIを使える

```rust
use rust_molkky::core::models::{ScoreBoard, Skittle};
use rust_molkky::core::services::calc_score;
let pin: rust_molkky::Skittle = Skittle::new(5);
let point = calc_score(&[pin]);
let board = ScoreBoard::new().add(point);
assert_eq!(board.value(), 5);
```

## ドメインサービスで進行中と勝者なし終了を区別する

```rust
use rust_molkky::core::models::{GameStatus, Molkky, ScoreBoard, Team};
use rust_molkky::core::services::{calc_score, judge_game};
let missed = calc_score(&[]);
let out = ScoreBoard::new().add(missed).add(missed).add(missed);
let teams = vec![Team::new("赤").with_score_board(out.clone()), Team::new("青")];
assert_eq!(judge_game(&teams), GameStatus::Won { team_index: 1 });
assert_eq!(Molkky::new(teams).status(), GameStatus::Won { team_index: 1 });
assert_eq!(judge_game(&[Team::new("赤").with_score_board(out)]), GameStatus::FinishedWithoutWinner);
```

## coreの公開APIに描画座標は含まれない

```compile_fail
use rust_molkky::core::Position;
```

```compile_fail
use rust_molkky::Skittle;
Skittle::new(5).position();
```
