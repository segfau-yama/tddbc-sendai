# 投てき・採点・得点履歴・ゲーム管理

## 採点して得点表へ記録する

```rust
use rust_molkky::{ScoreBoard, Skittle, Molkky, SimpleMolkky};
let first = SimpleMolkky.calc_score(&[Skittle::new(5)]);
let second = SimpleMolkky.calc_score(&[Skittle::new(11)]);
let mut board = ScoreBoard::default();
board = board.add(first);
board = board.add(second);
assert_eq!(first.value(), 5);
assert_eq!(board.total(), 16);
assert_eq!(board.history(), &[first, second]);
```

## Skittleの番号にNoneは指定できない

```compile_fail
use rust_molkky::Skittle;
Skittle::new(None);
```

## 採点入力とその要素にNoneは指定できない

```compile_fail
use rust_molkky::{Molkky, SimpleMolkky};
SimpleMolkky.calc_score(None);
```

```compile_fail
use rust_molkky::{Skittle, Molkky, SimpleMolkky};
SimpleMolkky.calc_score(&[None::<Skittle>]);
```

## 得点表にNoneは追加できない

```compile_fail
use rust_molkky::ScoreBoard;
ScoreBoard::default().add(None);
```

## 無効なPointを直接生成できない

```compile_fail
use rust_molkky::Point;
Point(99);
```

## 50点超過では履歴を残して25点へ戻す

```rust
use rust_molkky::ScoreState;
use rust_molkky::{ScoreBoard, Skittle, Molkky, SimpleMolkky};
let twelve = SimpleMolkky.calc_score(&[Skittle::new(12)]);
let three = SimpleMolkky.calc_score(&[Skittle::new(3)]);
let mut board = ScoreBoard::default();
for _ in 0..4 { board = board.add(twelve); }
board = board.add(three);
assert_eq!(board.total(), 25);
assert!(matches!(board.state(), ScoreState::Reset));
assert!(!matches!(board.state(), ScoreState::Won));
assert_eq!(board.history(), &[twelve, twelve, twelve, twelve, three]);
board = board.add(twelve);
assert_eq!(board.total(), 37);
assert!(!matches!(board.state(), ScoreState::Reset));
assert_eq!(board.history(), &[twelve, twelve, twelve, twelve, three, twelve]);
```

## 複数チームで50点ちょうどの勝者を判定する

```rust
use rust_molkky::{Name, Skittle, Team, Molkky, SimpleMolkky};
let mut teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let ten = SimpleMolkky.calc_score(&[Skittle::new(10)]);
for _ in 0..5 { teams = SimpleMolkky.record_point(&teams, 1, ten); }
assert_eq!(SimpleMolkky.judge_game(&teams), Some(1));
assert_eq!(teams[0].score_board().total(), 0);
assert_eq!(teams[1].score_board().total(), 50);
```

## 履歴と借用したチーム一覧は書き換えられない

```compile_fail
use rust_molkky::{ScoreBoard, Skittle, Molkky, SimpleMolkky};
let point = SimpleMolkky.calc_score(&[Skittle::new(5)]);
let board = ScoreBoard::default().add(point);
board.history()[0] = SimpleMolkky.calc_score(&[Skittle::new(12)]);
```

```compile_fail
use rust_molkky::{Name, Team, Molkky, SimpleMolkky};
let teams = vec![Team::new(Name::new("赤").unwrap())];
let borrowed: &[Team] = &teams;
borrowed[0] = SimpleMolkky.record_point(borrowed, 0, SimpleMolkky.calc_score(&[])).remove(0);
```

## 棒と対象個体と乱数源でデフォルトの投てきを行う

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Skittle, Stick, Molkky, SimpleMolkky};
use rust_molkky::simulation::ArrayField;
let mut rng = StdRng::seed_from_u64(42);
let pin = Skittle::new(5);
let field = ArrayField::new(vec![pin]);
let stick = Stick::new(1.0);
let fallen = SimpleMolkky.throw(&stick, field.skittles(), &mut rng);
assert_eq!(fallen, [pin]);
assert_eq!(SimpleMolkky.calc_score(&fallen).value(), 5);
assert_eq!(field.skittles(), [pin]);
```

## 投てき対象にNoneは指定できない

```compile_fail
use rust_molkky::{Molkky, SimpleMolkky, Stick};
SimpleMolkky.throw(&Stick::default(), None, &mut rand::rng());
```

## 投てき結果から採点と不変な状態の更新を組み合わせる

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Name, Field, Skittle, Stick, Team, Molkky, SimpleMolkky};
use rust_molkky::simulation::ArrayField;
let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let pin = Skittle::new(5);
let field = ArrayField::new(vec![pin]);
let stick = Stick::new(1.0);
let mut rng = StdRng::seed_from_u64(42);
let fallen = SimpleMolkky.throw(&stick, field.skittles(), &mut rng);
let updated = SimpleMolkky.record_point(&original, 0, SimpleMolkky.calc_score(&fallen));
assert_eq!(original[0].score_board().total(), 0);
assert_eq!(updated[0].score_board().total(), 5);
assert_eq!(field.skittles(), [pin]);
```

## 確率零と一の投てき結果

```rust
use rust_molkky::{Field, Skittle, Stick, Molkky, SimpleMolkky};
use rust_molkky::simulation::ArrayField;
let field = ArrayField::new(vec![Skittle::new(5), Skittle::new(8)]);
assert_eq!(Stick::default().fall_probability(), 0.25);
let zero = SimpleMolkky.throw(&Stick::new(0.0), field.skittles(), &mut rand::rng());
let all = SimpleMolkky.throw(&Stick::new(1.0), field.skittles(), &mut rand::rng());
assert!(zero.is_empty());
assert_eq!(all, field.skittles());
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
use rust_molkky::{ScoreBoard, Skittle, Molkky, SimpleMolkky};
let five = SimpleMolkky.calc_score(&[Skittle::new(5)]);
let eleven = SimpleMolkky.calc_score(&[Skittle::new(11)]);
let original = ScoreBoard::default().add(five);
let updated = original.add(eleven);
assert_eq!(original.history(), &[five]);
assert_eq!(updated.history(), &[five, eleven]);
```

## チームは得点表を差し替えて新しい値を返す

```rust
use rust_molkky::{Name, ScoreBoard, Skittle, Team, Molkky, SimpleMolkky};
let original = Team::new(Name::new("赤").unwrap());
let point = SimpleMolkky.calc_score(&[Skittle::new(5)]);
let updated = original.with_score_board(ScoreBoard::default().add(point));
assert_eq!(original.score_board().total(), 0);
assert_eq!(updated.score_board().total(), 5);
assert_eq!(original.name(), updated.name());
```

## 3回連続0点の失格後も残ったチームで継続する

```rust
use rust_molkky::{Name, Skittle, Team, Molkky, SimpleMolkky};
let mut teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap()), Team::new(Name::new("緑").unwrap())];
let missed = SimpleMolkky.calc_score(&[]);
for _ in 0..3 { teams = SimpleMolkky.record_point(&teams, 0, missed); }
assert_eq!(SimpleMolkky.judge_game(&teams), None);
assert_eq!(SimpleMolkky.next_team(&teams, 2), Some(1));
teams = SimpleMolkky.record_point(&teams, 1, SimpleMolkky.calc_score(&[Skittle::new(5)]));
assert_eq!(teams[1].score_board().total(), 5);
for _ in 0..3 { teams = SimpleMolkky.record_point(&teams, 1, missed); }
assert_eq!(SimpleMolkky.judge_game(&teams), Some(2));
assert_eq!(SimpleMolkky.next_team(&teams, 1), None);
assert_eq!(teams[2].score_board().total(), 0);
```

## 同番号の別個体を配置し採点できる

```rust
use rust_molkky::{Field, Skittle, Molkky, SimpleMolkky};
use rust_molkky::simulation::ArrayField;
let first = Skittle::new(12);
let second = Skittle::new(12);
assert_ne!(first.id(), second.id());
let field = ArrayField::new(vec![first, second]);
assert_eq!(SimpleMolkky.calc_score(field.skittles()).value(), 2);
assert_eq!(SimpleMolkky.calc_score(&[first]).value(), 12);
```

## チーム内部の得点表は共有参照から差し替えられない

```compile_fail
use rust_molkky::{Name, ScoreBoard, Team};
let team = Team::new(Name::new("赤").unwrap());
*team.score_board() = ScoreBoard::default();
```

## coreモジュールからも同じ公開APIを使える

```rust
use rust_molkky::core::models::{ScoreBoard, Skittle};
use rust_molkky::core::services::{Molkky, SimpleMolkky};
let pin: rust_molkky::Skittle = Skittle::new(5);
let point = SimpleMolkky.calc_score(&[pin]);
let board = ScoreBoard::default().add(point);
assert_eq!(board.total(), 5);
```

## チーム状態から勝者と継続可否を判定する

```rust
use rust_molkky::core::models::{Name, ScoreBoard, Team};
use rust_molkky::core::services::{Molkky, SimpleMolkky};
let missed = SimpleMolkky.calc_score(&[]);
let out = ScoreBoard::default().add(missed).add(missed).add(missed);
let teams = vec![Team::new(Name::new("赤").unwrap()).with_score_board(out.clone()), Team::new(Name::new("青").unwrap())];
assert_eq!(SimpleMolkky.judge_game(&teams), Some(1));
let lost = [Team::new(Name::new("赤").unwrap()).with_score_board(out)];
assert_eq!(SimpleMolkky.judge_game(&lost), None);
assert_eq!(SimpleMolkky.next_team(&lost, 0), None);
let live = [Team::new(Name::new("赤").unwrap())];
assert_eq!(SimpleMolkky.judge_game(&live), None);
assert_eq!(SimpleMolkky.next_team(&live, 0), Some(0));
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
use rust_molkky::{Name, Team, Molkky, SimpleMolkky};
let first = Team::new(Name::new("赤").unwrap());
let second = Team::new(Name::new("赤").unwrap());
assert_ne!(first.id(), second.id());
let id = first.id();
let original = vec![first, second];
let updated = SimpleMolkky.record_point(&original, 0, SimpleMolkky.calc_score(&[]));
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

## 座標付きFieldからも個体集合だけを渡して投げられる

```rust
use rust_molkky::{Field, FieldGenerator, Skittle, Stick, Molkky, SimpleMolkky};
struct Board { pins: Vec<Skittle>, x: f64 }
impl Field for Board {
    fn skittles(&self) -> &[Skittle] { &self.pins }
}
struct Setup;
impl FieldGenerator for Setup {
    type Field = Board;
    fn generate(&self) -> Board { Board { pins: vec![Skittle::new(12), Skittle::new(12)], x: 0.0 } }
}
let field = Setup.generate();
let fallen = SimpleMolkky.throw(&Stick::new(1.0), field.skittles(), &mut rand::rng());
assert_eq!(fallen, field.skittles());
assert_eq!(field.x, 0.0);
assert_eq!(SimpleMolkky.calc_score(&fallen).value(), 2);
```

## 空のMolkky実装でデフォルトのルールを利用する

```rust
use rust_molkky::{Molkky, Name, Skittle, Team};
struct MyMolkky;
impl Molkky for MyMolkky {}
let rules: &dyn Molkky = &MyMolkky;
let original = vec![Team::new(Name::new("赤").unwrap())];
let updated = rules.record_point(&original, 0, rules.calc_score(&[Skittle::new(12)]));
assert_eq!(updated[0].score_board().total(), 12);
assert_eq!(original[0].score_board().total(), 0);
assert_eq!(rules.judge_game(&updated), None);
```

## 履歴から状態を確定し補正後の合計を取得する

```rust
use rust_molkky::{Molkky, ScoreBoard, ScoreState, SimpleMolkky, Skittle};
let history = [12,12,12,12,3].map(|n| SimpleMolkky.calc_score(&[Skittle::new(n)])).to_vec();
let original = ScoreBoard::new(history);
assert_eq!(original.total(), 25);
assert_eq!(original.state(), ScoreState::Reset);
let updated = original.add(SimpleMolkky.calc_score(&[]));
assert_eq!(updated.state(), ScoreState::Calculated);
assert_eq!(original.state(), ScoreState::Reset);
let won = ScoreBoard::new(vec![SimpleMolkky.calc_score(&[Skittle::new(10)]); 5]);
assert_eq!(won.state(), ScoreState::Won);
assert!(std::panic::catch_unwind(|| won.add(SimpleMolkky.calc_score(&[]))).is_err());
```

## 状態を直接変更して履歴との整合性を壊すことはできない

```compile_fail
use rust_molkky::{ScoreBoard, ScoreState};
let mut board = ScoreBoard::default();
board.state = ScoreState::Won;
```
