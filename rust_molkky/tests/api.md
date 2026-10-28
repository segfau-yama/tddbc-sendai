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

## Strategyから入力を変更せず投てき結果を得る

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Skittle, calc_score, throw_stick};
use rust_molkky::simulation::{ArrayField, RandomThrow, Stick};
let mut rng = StdRng::seed_from_u64(42);
let pin = Skittle::new(5);
let field = ArrayField::new(vec![pin]);
let stick = Stick::new(1.0);
let outcome = throw_stick(&field, &mut RandomThrow::new(&stick, &mut rng)).unwrap();
assert_eq!(outcome.knocked_over(), [pin]);
assert_eq!(calc_score(outcome.knocked_over()).value(), 5);
assert_eq!(field.skittles(), [pin]);
assert_eq!(outcome.field().skittles(), [pin]);
```

## 投てき対象にNoneは指定できない

```compile_fail
use rust_molkky::throw_stick;
use rust_molkky::simulation::{RandomThrow, Stick};
let stick = Stick::default();
let mut rng = rand::rng();
let mut strategy = RandomThrow::new(&stick, &mut rng);
throw_stick(None, &mut strategy);
```

## 投てき結果から採点と不変な状態の更新を組み合わせる

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Name, Field, Skittle, Team, calc_score, record_point, throw_stick};
use rust_molkky::simulation::{ArrayField, RandomThrow, Stick};
let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let pin = Skittle::new(5);
let field = ArrayField::new(vec![pin]);
let stick = Stick::new(1.0);
let mut rng = StdRng::seed_from_u64(42);
let outcome = throw_stick(&field, &mut RandomThrow::new(&stick, &mut rng)).unwrap();
let updated = record_point(&original, 0, calc_score(outcome.knocked_over()));
assert_eq!(original[0].score_board().value(), 0);
assert_eq!(updated[0].score_board().value(), 5);
assert_eq!(field.skittles(), [pin]);
```

## 確率方式を具体的なStrategyとして選択する

```rust
use rust_molkky::{Field, Skittle, throw_stick};
use rust_molkky::simulation::{ArrayField, RandomThrow, Stick};
let field = ArrayField::new(vec![Skittle::new(5), Skittle::new(8)]);
assert_eq!(Stick::default().fall_probability(), 0.25);
let zero = throw_stick(&field, &mut RandomThrow::new(&Stick::new(0.0), &mut rand::rng())).unwrap();
let all = throw_stick(&field, &mut RandomThrow::new(&Stick::new(1.0), &mut rand::rng())).unwrap();
assert!(zero.knocked_over().is_empty());
assert_eq!(all.knocked_over(), field.skittles());
```

## Fieldの共有参照からピン集合を書き換えることはできない

```compile_fail
use rust_molkky::{Field, FieldGenerator, Skittle};
use rust_molkky::simulation::StandardFieldGenerator;
let field = StandardFieldGenerator.generate();
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

## 同番号の別個体を配置し採点できる

```rust
use rust_molkky::{Field, Skittle, calc_score};
use rust_molkky::simulation::ArrayField;
let first = Skittle::new(12);
let second = Skittle::new(12);
assert_ne!(first.id(), second.id());
let field = ArrayField::new(vec![first, second]);
assert_eq!(calc_score(field.skittles()).value(), 2);
assert_eq!(calc_score(&[first]).value(), 12);
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

## 座標付きFieldと独自の一投をStrategyとして実装する

```rust
use rust_molkky::{Field, FieldGenerator, Skittle, ThrowOutcome, ThrowStrategy, throw_stick};
#[derive(Clone)]
struct Board { pins: Vec<Skittle>, x: f64 }
impl Field for Board {
    fn skittles(&self) -> &[Skittle] { &self.pins }
}
struct Setup;
impl FieldGenerator for Setup {
    type Field = Board;
    fn generate(&self) -> Board { Board { pins: vec![Skittle::new(12), Skittle::new(12)], x: 0.0 } }
}
// 座標更新の接続例であり、物理シミュレーションではない。
struct Move;
impl ThrowStrategy<Board> for Move {
    type Error = std::convert::Infallible;
    fn throw(&mut self, field: &Board) -> Result<ThrowOutcome<Board>, Self::Error> {
        let mut updated = field.clone();
        updated.x += 1.0;
        Ok(ThrowOutcome::new(updated, vec![field.pins[0]]))
    }
}
let field = Setup.generate();
let strategy: &mut dyn ThrowStrategy<Board, Error = std::convert::Infallible> = &mut Move;
let result = throw_stick(&field, strategy).unwrap();
assert_eq!(result.field().x, 1.0);
assert_eq!(field.x, 0.0);
assert_eq!(result.knocked_over(), [field.pins[0]]);
```
