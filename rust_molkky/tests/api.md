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
use rust_molkky::{Name, GameStatus, Skittle, Team, calc_score, judge_game, record_point};
let mut teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let ten = calc_score(&[Skittle::new(10)]);
for _ in 0..5 { teams = record_point(&teams, 1, ten); }
assert_eq!(judge_game(&teams), GameStatus::Finished { winner: Some(1) });
assert_eq!(teams[0].score_board().value(), 0);
assert_eq!(teams[1].score_board().value(), 50);
```

## 履歴と借用したチーム一覧は書き換えられない

```compile_fail
use rust_molkky::{ScoreBoard, Skittle, calc_score};
let point = calc_score(&[Skittle::new(5)]);
let board = ScoreBoard::new().add(point);
board.history()[0] = calc_score(&[Skittle::new(12)]);
```

```compile_fail
use rust_molkky::{Name, Team, calc_score, record_point};
let teams = vec![Team::new(Name::new("赤").unwrap())];
let borrowed: &[Team] = &teams;
borrowed[0] = record_point(borrowed, 0, calc_score(&[])).remove(0);
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
use rust_molkky::{Name, Field, Skittle, Stick, Team, calc_score, record_point, throw_stick};
let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let field = Field::new(vec![Skittle::new(5)]);
let stick = Stick::new(1.0);
let mut rng = StdRng::seed_from_u64(42);
let knocked_over = throw_stick(&stick, field.skittles(), &mut rng);
let updated = record_point(&original, 0, calc_score(&knocked_over));
assert_eq!(original[0].score_board().value(), 0);
assert_eq!(updated[0].score_board().value(), 5);
assert_eq!(field.skittles(), [Skittle::new(5)]);
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
use rust_molkky::{Name, ScoreBoard, Skittle, Team, calc_score};
let original = Team::new(Name::new("赤").unwrap());
let point = calc_score(&[Skittle::new(5)]);
let updated = original.with_score_board(ScoreBoard::new().add(point));
assert_eq!(original.score_board().value(), 0);
assert_eq!(updated.score_board().value(), 5);
assert_eq!(original.name(), updated.name());
```

## 3回連続0点の失格後も残ったチームで継続する

```rust
use rust_molkky::{Name, GameStatus, Skittle, Team, calc_score, judge_game, next_team, record_point};
let mut teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap()), Team::new(Name::new("緑").unwrap())];
let missed = calc_score(&[]);
for _ in 0..3 { teams = record_point(&teams, 0, missed); }
assert_eq!(judge_game(&teams), GameStatus::InProgress);
assert_eq!(next_team(&teams, 2), Some(1));
teams = record_point(&teams, 1, calc_score(&[Skittle::new(5)]));
assert_eq!(teams[1].score_board().value(), 5);
for _ in 0..3 { teams = record_point(&teams, 1, missed); }
assert_eq!(judge_game(&teams), GameStatus::Finished { winner: Some(2) });
assert_eq!(next_team(&teams, 1), None);
assert_eq!(teams[2].score_board().value(), 0);
```

## FieldとStickは採点データと独立して保持する

```rust
use rust_molkky::{Name, Field, Skittle, Stick, Team, calc_score, record_point};
let teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let field = Field::new(vec![Skittle::new(5)]);
let stick = Stick::new(0.75);
let recorded = record_point(&teams, 1, calc_score(&[Skittle::new(5)]));
assert_eq!(field.skittles(), [Skittle::new(5)]);
assert_eq!(stick.fall_probability(), 0.75);
assert_eq!(teams[1].score_board().value(), 0);
assert_eq!(recorded[1].score_board().value(), 5);
```

## Pointの記録結果を破棄できない

```compile_fail
#![deny(unused_must_use)]
use rust_molkky::{Name, Team, calc_score, record_point};
let teams = vec![Team::new(Name::new("赤").unwrap())];
record_point(&teams, 0, calc_score(&[]));
```

## チーム内部の得点表は共有参照から差し替えられない

```compile_fail
use rust_molkky::{Name, ScoreBoard, Team};
let team = Team::new(Name::new("赤").unwrap());
*team.score_board() = ScoreBoard::new();
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
use rust_molkky::core::models::{GameStatus, Name, ScoreBoard, Team};
use rust_molkky::core::services::{calc_score, judge_game};
let missed = calc_score(&[]);
let out = ScoreBoard::new().add(missed).add(missed).add(missed);
let teams = vec![Team::new(Name::new("赤").unwrap()).with_score_board(out.clone()), Team::new(Name::new("青").unwrap())];
assert_eq!(judge_game(&teams), GameStatus::Finished { winner: Some(1) });
assert_eq!(judge_game(&[Team::new(Name::new("赤").unwrap()).with_score_board(out)]), GameStatus::Finished { winner: None });
```

## coreの公開APIに描画座標は含まれない

```compile_fail
use rust_molkky::core::Position;
```

```compile_fail
use rust_molkky::Skittle;
Skittle::new(5).position();
```

## チームの識別子は名前や得点更新で変わらない

```rust
use rust_molkky::{Name, Team, calc_score, record_point};
let first = Team::new(Name::new("赤").unwrap());
let second = Team::new(Name::new("赤").unwrap());
assert_ne!(first.id(), second.id());
let id = first.id();
let original = vec![first, second];
let updated = record_point(&original, 0, calc_score(&[]));
assert_eq!(updated[0].id(), id);
assert_eq!(updated[0].name().as_str(), "赤");
```

## 不正な名前はチームを生成する前に拒否する

```rust
use rust_molkky::{Name, NameError};
assert_eq!(Name::new("　"), Err(NameError::Empty));
assert_eq!(Name::new("赤\n青"), Err(NameError::ControlCharacter));
assert_eq!(Name::new(&"赤".repeat(31)), Err(NameError::TooLong));
assert_eq!(Name::new(" 赤 ").unwrap().as_str(), "赤");
```

## 未検証の文字列やUUIDを直接フィールドへ入れられない

```compile_fail
use rust_molkky::Team;
Team::new("赤");
```

```compile_fail
use rust_molkky::Name;
Name(String::new());
```

```compile_fail
use rust_molkky::Id;
Id(uuid::Uuid::nil());
```
