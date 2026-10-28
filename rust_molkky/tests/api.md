# 投てき・採点・得点履歴・ゲーム管理

## 採点して得点表へ記録する

```rust
use rust_molkky::Skittles;
use rust_molkky::{ScoreBoard, Skittle};
let first = Skittles::new(&[Skittle::new(5)]).calc_score();
let second = Skittles::new(&[Skittle::new(11)]).calc_score();
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
use rust_molkky::Skittles;
use rust_molkky::{SimpleMolkky};
Skittles::new(None).calc_score();
```

```compile_fail
use rust_molkky::Skittles;
use rust_molkky::{Skittle};
Skittles::new(&[None::<Skittle>]).calc_score();
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

## 50点超過では履歴を25点の基準値へ置き換える

```rust
use rust_molkky::Skittles;
use rust_molkky::ScoreState;
use rust_molkky::{ScoreBoard, Skittle};
let twelve = Skittles::new(&[Skittle::new(12)]).calc_score();
let three = Skittles::new(&[Skittle::new(3)]).calc_score();
let mut board = ScoreBoard::default();
for _ in 0..4 { board = board.add(twelve); }
board = board.add(three);
assert_eq!(board.total(), 25);
assert!(matches!(board.state(), ScoreState::Reset));
assert!(!matches!(board.state(), ScoreState::Won));
assert_eq!(board.history().iter().map(|point| point.value()).collect::<Vec<_>>(), [25]);
board = board.add(twelve);
assert_eq!(board.total(), 37);
assert!(!matches!(board.state(), ScoreState::Reset));
assert_eq!(board.history().iter().map(|point| point.value()).collect::<Vec<_>>(), [25, 12]);
```

## 使用側で得点を更新しWonのチームを受け取る

```rust
use rust_molkky::Skittles;
use rust_molkky::Teams;
use rust_molkky::{Name, Skittle, Team};
let mut teams = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let ten = Skittles::new(&[Skittle::new(10)]).calc_score();
for _ in 0..5 {
    let board = teams[1].score_board().add(ten);
    teams[1] = teams[1].with_score_board(board);
}
let winner = Teams::new(&teams).judge_game().unwrap();
assert_eq!(winner.id(), teams[1].id());
assert_eq!(teams[0].score_board().total(), 0);
assert_eq!(teams[1].score_board().total(), 50);
```

## 履歴と借用したチーム一覧は書き換えられない

```compile_fail
use rust_molkky::Skittles;
use rust_molkky::{ScoreBoard, Skittle};
let point = Skittles::new(&[Skittle::new(5)]).calc_score();
let board = ScoreBoard::default().add(point);
board.history()[0] = Skittles::new(&[Skittle::new(12)]).calc_score();
```

```compile_fail
use rust_molkky::Skittles;
use rust_molkky::{Name, Team};
let teams = vec![Team::new(Name::new("赤").unwrap())];
let borrowed: &[Team] = &teams;
borrowed[0] = borrowed[0].with_score_board(borrowed[0].score_board().add(Skittles::new(&[]).calc_score()));
```

## 棒と対象個体と判定関数でhitを行う

```rust
use rust_molkky::Skittles;
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Skittle, SkittleState, Stick, Molkky, SimpleMolkky};
use rust_molkky::simulation::{ArrayField, RandomStick};
let mut rng = StdRng::seed_from_u64(42);
let pin = Skittle::new(5);
let field = ArrayField::new(Field::new(), vec![pin]);
let stick = RandomStick::new(Stick::new(), 1.0);
let updated = SimpleMolkky.hit(stick.stick(), field.skittles(), &mut |_, pins| stick.judge(pins, &mut rng));
assert_eq!(updated, [pin.with_state(SkittleState::Fall)]);
let fallen: Vec<_> = updated.into_iter().filter(|pin| pin.state() == SkittleState::Fall).collect();
assert_eq!(Skittles::new(&fallen).calc_score().value(), 5);
assert_eq!(field.skittles(), [pin]);
```

## hitの対象にNoneは指定できない

```compile_fail
use rust_molkky::{Molkky, SimpleMolkky, Stick};
SimpleMolkky.hit(&Stick::default(), None, &mut |_, pins| pins.to_vec());
```

## hitの結果から採点と不変な状態の更新を組み合わせる

```rust
use rust_molkky::Skittles;
use rust_molkky::{Name, Skittle, SkittleState, Stick, Team, Molkky, SimpleMolkky};
let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let pins = [Skittle::new(5), Skittle::new(12)];
let updated_pins = SimpleMolkky.hit(&Stick::new(), &pins, &mut |_, pins| {
    vec![pins[0].with_state(SkittleState::Fall), pins[1]]
});
let fallen: Vec<_> = updated_pins.into_iter().filter(|pin| pin.state() == SkittleState::Fall).collect();
let board = original[0].score_board().add(Skittles::new(&fallen).calc_score());
let updated = original[0].with_score_board(board);
assert_eq!(original[0].score_board().total(), 0);
assert_eq!(updated.score_board().total(), 5);
assert!(pins.iter().all(|pin| pin.state() == SkittleState::Stand));
```

## 確率零と一の判定は全個体の新しい状態を返す

```rust
use rust_molkky::{Skittle, SkittleState, Stick};
use rust_molkky::simulation::RandomStick;
let pins = [Skittle::new(5), Skittle::new(8)];
assert_eq!(RandomStick::default().fall_probability(), 0.25);
let zero = RandomStick::new(Stick::new(), 0.0).judge(&pins, &mut rand::rng());
let all = RandomStick::new(Stick::new(), 1.0).judge(&pins, &mut rand::rng());
assert_eq!(zero, pins);
assert_eq!(all.len(), pins.len());
assert!(all.iter().all(|pin| pin.state() == SkittleState::Fall));
```

## Fieldの共有参照からピン集合を書き換えることはできない

```compile_fail
use rust_molkky::Skittle;
use rust_molkky::simulation::StandardFieldGenerator;
let field = StandardFieldGenerator.generate();
field.skittles()[0] = Skittle::new(12);
```

## 得点表の更新は元の履歴を変更しない

```rust
use rust_molkky::Skittles;
use rust_molkky::{ScoreBoard, Skittle};
let five = Skittles::new(&[Skittle::new(5)]).calc_score();
let eleven = Skittles::new(&[Skittle::new(11)]).calc_score();
let original = ScoreBoard::default().add(five);
let updated = original.add(eleven);
assert_eq!(original.history(), &[five]);
assert_eq!(updated.history(), &[five, eleven]);
```

## チームは得点表を差し替えて新しい値を返す

```rust
use rust_molkky::Skittles;
use rust_molkky::{Name, ScoreBoard, Skittle, Team};
let original = Team::new(Name::new("赤").unwrap());
let point = Skittles::new(&[Skittle::new(5)]).calc_score();
let updated = original.with_score_board(ScoreBoard::default().add(point));
assert_eq!(original.score_board().total(), 0);
assert_eq!(updated.score_board().total(), 5);
assert_eq!(original.name(), updated.name());
```

## 手番の切り替えと失格者のスキップはnext_teamで行う

```rust
use rust_molkky::Skittles;
use rust_molkky::Teams;
use rust_molkky::{Name, ScoreBoard, Team};
let missed = Skittles::new(&[]).calc_score();
let red = Team::new(Name::new("赤").unwrap()).with_turn(true);
let red = red.with_score_board(ScoreBoard::new(vec![missed; 3]));
let teams = [red, Team::new(Name::new("青").unwrap()), Team::new(Name::new("緑").unwrap())];
let next = Teams::new(&teams).next_team();
assert!(!next[0].turn());
assert!(next[1].turn());
assert!(!next[2].turn());
assert!(teams[0].turn());
assert_eq!(next[1].id(), teams[1].id());
```

## 同番号の別個体を配置し採点できる

```rust
use rust_molkky::Skittles;
use rust_molkky::{Field, Skittle};
use rust_molkky::simulation::ArrayField;
let first = Skittle::new(12);
let second = Skittle::new(12);
assert_ne!(first.id(), second.id());
let field = ArrayField::new(Field::new(), vec![first, second]);
assert_eq!(Skittles::new(field.skittles()).calc_score().value(), 2);
assert_eq!(Skittles::new(&[first]).calc_score().value(), 12);
```

## チーム内部の得点表は共有参照から差し替えられない

```compile_fail
use rust_molkky::{Name, ScoreBoard, Team};
let team = Team::new(Name::new("赤").unwrap());
*team.score_board() = ScoreBoard::default();
```

## coreモジュールからも同じ公開APIを使える

```rust
use rust_molkky::Skittles;
use rust_molkky::core::models::{ScoreBoard, Skittle};
use rust_molkky::core::services::{SimpleMolkky};
let pin: rust_molkky::Skittle = Skittle::new(5);
let point = Skittles::new(&[pin]).calc_score();
let board = ScoreBoard::default().add(point);
assert_eq!(board.total(), 5);
```

## 手番と勝者は異なるチームでも個体として取得できる

```rust
use rust_molkky::Skittles;
use rust_molkky::Teams;
use rust_molkky::{Name, ScoreBoard, Skittle, Team};
let won = ScoreBoard::new(vec![Skittles::new(&[Skittle::new(10)]).calc_score(); 5]);
let teams = [
    Team::new(Name::new("赤").unwrap()).with_turn(true),
    Team::new(Name::new("青").unwrap()).with_score_board(won),
];
assert_eq!(teams.iter().find(|team| team.turn()).unwrap().id(), teams[0].id());
assert!(Teams::new(&teams).next_team().iter().all(|team| !team.turn()));
assert_eq!(Teams::new(&teams).judge_game().unwrap().id(), teams[1].id());
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
use rust_molkky::Skittles;
use rust_molkky::{Name, Team};
let first = Team::new(Name::new("赤").unwrap());
let second = Team::new(Name::new("赤").unwrap());
assert_ne!(first.id(), second.id());
let id = first.id();
let original = vec![first, second];
let updated = original[0].with_score_board(original[0].score_board().add(Skittles::new(&[]).calc_score()));
assert_eq!(updated.id(), id);
assert_eq!(updated.name().as_str(), "赤");
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

## Fieldの識別子と座標付き表現を合成する

```rust
use rust_molkky::Skittles;
use rust_molkky::{Field, Skittle, SkittleState, Stick, Molkky, SimpleMolkky};
struct Board { field: Field, pins: Vec<Skittle>, positions: Vec<(f64, f64)> }
let identity = Field::new();
let field = Board { field: identity, pins: vec![Skittle::new(12), Skittle::new(12)], positions: vec![(0.0, 0.0), (1.0, 0.0)] };
let updated = SimpleMolkky.hit(&Stick::new(), &field.pins, &mut |_, pins| {
    pins.iter().map(|pin| pin.with_state(SkittleState::Fall)).collect()
});
assert_eq!(field.field.id(), identity.id());
assert_eq!(field.positions, [(0.0, 0.0), (1.0, 0.0)]);
assert_eq!(Skittles::new(&updated).calc_score().value(), 2);
```

## 空のMolkky実装でデフォルトのルールを利用する

```rust
use rust_molkky::Skittles;
use rust_molkky::Teams;
use rust_molkky::{Molkky, Name, Skittle, Team};
struct MyMolkky;
impl Molkky for MyMolkky {}
let rules: &dyn Molkky = &MyMolkky;
let original = vec![Team::new(Name::new("赤").unwrap())];
let pins = rules.hit(&rust_molkky::Stick::new(), &[Skittle::new(12)], &mut |_, pins| pins.to_vec());
let board = original[0].score_board().add(Skittles::new(&pins).calc_score());
let updated = original[0].with_score_board(board);
assert_eq!(updated.score_board().total(), 12);
assert_eq!(original[0].score_board().total(), 0);
assert!(Teams::new(&[updated]).judge_game().is_none());
```

## 履歴から状態を確定し補正後の合計を取得する

```rust
use rust_molkky::Skittles;
use rust_molkky::{ScoreBoard, ScoreState, Skittle};
let history = [12,12,12,12,3].map(|n| Skittles::new(&[Skittle::new(n)]).calc_score()).to_vec();
let original = ScoreBoard::new(history);
assert_eq!(original.total(), 25);
assert_eq!(original.state(), ScoreState::Reset);
let updated = original.add(Skittles::new(&[]).calc_score());
assert_eq!(updated.state(), ScoreState::Calculated);
assert_eq!(original.state(), ScoreState::Reset);
let won = ScoreBoard::new(vec![Skittles::new(&[Skittle::new(10)]).calc_score(); 5]);
assert_eq!(won.state(), ScoreState::Won);
assert!(std::panic::catch_unwind(|| won.add(Skittles::new(&[]).calc_score())).is_err());
```

## 状態を直接変更して履歴との整合性を壊すことはできない

```compile_fail
use rust_molkky::{ScoreBoard, ScoreState};
let mut board = ScoreBoard::default();
board.state = ScoreState::Won;
```

## 手番変更でもIdと元のチームを保つ

```rust
use rust_molkky::Teams;
use rust_molkky::{Name, Team};
let original = Team::new(Name::new("赤").unwrap());
let active = original.with_turn(true);
assert!(!original.turn());
assert!(active.turn());
assert_eq!(active.id(), original.id());
let teams = [active];
let next = Teams::new(&teams).next_team();
assert_eq!(next[0].id(), original.id());
assert!(next[0].turn());
```

## 判定関数の型別名はクロージャの可変借用として使える

```rust
use rust_molkky::{JudgeStrategy, Molkky, SimpleMolkky, Skittle, Stick};
let mut count = 0;
let mut judge = |_: &Stick, pins: &[Skittle]| { count += 1; pins.to_vec() };
let strategy: JudgeStrategy<'_> = &mut judge;
let rules: &dyn Molkky = &SimpleMolkky;
assert!(rules.hit(&Stick::new(), &[], strategy).is_empty());
assert_eq!(count, 1);
```

## 識別子だけの棒に確率を設定することはできない

```compile_fail
use rust_molkky::Stick;
Stick::new(0.25);
```

## Skittlesは同番号の別個体を許可し元の一覧を借用する

```rust
use rust_molkky::Skittles;
use rust_molkky::Skittle;
let pins = [Skittle::new(12), Skittle::new(12)];
let skittles = Skittles::new(&pins);
skittles.validate();
assert_eq!(skittles.calc_score().value(), 2);
```

## Skittlesが借用している間は元の一覧を書き換えられない

```compile_fail
use rust_molkky::{Skittle, Skittles};
let mut pins = [Skittle::new(5)];
let skittles = Skittles::new(&pins);
pins[0] = Skittle::new(12);
skittles.validate();
```
