# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

`rust_molkky`は、Java版を起点にチーム・ゲーム管理と距離に応じた投てきモデルを追加したライブラリとTUIです。
実装は`src/`、振る舞いのテストは`tests/`にあります。依存取得にはインターネット接続が必要です。

```sh
cd rust_molkky
cargo test --locked
cargo build --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- 赤チーム 青チーム  # 対話型ターミナルで起動
```

TUIではSpace / Enterで投てき、矢印キーで投てき位置を移動、Rで再開、Q / Escで終了します。

| 責務 | 公開API | 実装 |
| --- | --- | --- |
| 番号1〜12と位置を持つピン | `Skittle`, `Position` | `src/skittle.rs` |
| 現在のモデルで使う棒の投てき位置 | `Stick` | `src/stick.rs` |
| 倒れたピンの判定 | `throw_stick` | `src/throw.rs` |
| 1投の採点と点数の値 | `calc_score`, `Point` | `src/score.rs` |
| 1チームの点数履歴と現在得点 | `ScoreBoard` | `src/score.rs` |
| 名称と得点表 | `Team` | `src/team.rs` |
| 複数チーム・共通のFieldとStick・終了判定 | `Molkky` | `src/molkky.rs` |
| ピンの配置 | `Field` | `src/field.rs` |
| 元の配置を保ち、移動後の配置を返す | `relocate_skittles` | `src/movement.rs` |
| 棒とピンの位置関係から確率を求める | `fall_probability` | `src/throw.rs` |

採点と履歴更新を分離します。`calc_score`は入力だけから`Point`を返す純粋関数です。
0本なら0点、1本なら番号、複数本なら本数です。`ScoreBoard::add`は元の得点表を変えず、その点数を履歴へ追加した新しい得点表を返します。

```rust
use rust_molkky::{ScoreBoard, Skittle, calc_score};

let first = calc_score(&[Skittle::new(5), Skittle::new(8)]);
let second = calc_score(&[Skittle::new(11)]);
let mut board = ScoreBoard::new();
board = board.add(first);
board = board.add(second);
assert_eq!(first.value(), 2);
assert_eq!(board.value(), 13);
assert_eq!(board.history(), &[first, second]);
```

- `Point`は読み取り専用で、`calc_score`が0〜12の範囲を保証します。
- `ScoreBoard`の`score_history`は非公開の`Vec<Point>`です。`history()`では読み取り専用のスライスを返します。
- 履歴は生の点数の順序を保持します。現在得点は投てき順に計算し、50点超過ならその回に25点へ戻ります。
- `Skittle::new`は番号1〜12を保証します。同じ番号のピンは、位置が違ってもField・投てき対象・採点入力で拒否します。
- 不正入力は既存方針の`panic!`で拒否します。通常のRustの値・参照・スライスにnullはありません。

`throw_stick`は棒と対象のピン、呼び出し側の乱数源を受け取り、倒れたピンのコピーを返します。
入力のピンは移動しませんが、乱数源は更新するため厳密な純粋関数ではありません。
`Stick`は現在の距離モデルに必要な位置だけを持ち、速度・角度・衝突モデルは追加していません。

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Position, Skittle, Stick, throw_stick, calc_score};

let mut rng = StdRng::seed_from_u64(42);
let targets = [Skittle::new(5).with_position(Position::new(0, 0))];
let knocked_over = throw_stick(&Stick::new(Position::new(0, 0)), &targets, &mut rng);
assert_eq!(knocked_over, targets); // 同じ位置のピンは必ず倒れる。
assert_eq!(calc_score(&knocked_over).value(), 5);
```

ゲームへは採点済みのPointを記録します。ピンの判定・採点はTeamやMolkkyの責務に含めません。

```rust
use rust_molkky::{Molkky, Skittle, Team, calc_score};

let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let twelve = calc_score(&[Skittle::new(12)]);
for _ in 0..4 { game = game.record_point(0, twelve); }
game = game.record_point(0, calc_score(&[Skittle::new(3)])); // 48 + 3で25点へ。
assert_eq!(game.teams()[0].score_board().value(), 25);
game = game.record_point(0, twelve); // 37点。
game = game.record_point(0, twelve); // 49点。
game = game.record_point(0, calc_score(&[Skittle::new(1)]));
assert_eq!(game.result(), Some(0)); // 50点ちょうどで赤チームの勝利。
```

- `Molkky::record_point`は元の値を変えず、点数を記録した新しいゲームを返します。TeamはPointに依存せず、`with_score_board`で得点表を差し替えた新しいTeamを返します。
- 得点・勝利・直近の超過補正・失格はScoreBoardの`value()`・`is_winner()`・`was_reset()`・`is_disqualified()`で取得します。TUIとMolkkyは採点側の判定を使います。
- `result()`は50点ちょうどのチーム、または2チーム以上の参加で最後まで失格せず残った1チームをSome(index)で返します。勝者がいなければNoneです。`is_finished()`は勝者がいる場合、または全チームが失格した場合にtrueを返します。
- 終了後・存在しないチーム・失格チームへの記録は、履歴を変える前に拒否します。`teams()`から直接更新はできません。
- チームは1件以上必要です。空の名称・同名は許容し、インデックスで識別します。構築時は採点側で履歴を再計算し、25点補正後の得点を検証します。複数勝者、50点到達後や失格後に追加記録がある履歴を拒否します。
- 1チームが3回連続0点なら失格です。正の点数で連続回数は途切れます。失格した投てきまでの履歴は保持します。2チーム以上で始めたゲームは、残り1チームになればそのチームの勝利です。最初から1チームなら50点を目指し、失格すれば勝者なしで終了します。
- ライブラリは手番を強制しません。TUIは失格チームを飛ばして手番を切り替え、全チーム失格時には終了を表示します。

Molkkyは全チーム共通のFieldとStickを1つずつ所有します。Teamが持つのは名称とScoreBoardです。
`field()`・`stick()`は読み取り専用で、`with_field`・`with_stick`は元のゲームを変えずに新しいゲームを返します。
`Molkky::new`はFieldを既定の12本、Stickを原点に初期化します。TUIはこの共通状態だけを参照します。
投てきの判定と配置の更新は呼び出し側で組み合わせます。`simulation.rs`と`random_throw`は削除しました。
`relocate_skittles(&Field, &mut impl Rng)`は元の配置を変えず、全ピンを移動した新しいFieldを返します。Fieldは配置と検証だけを担当します。

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{Field, Molkky, Position, Stick, Team, calc_score, relocate_skittles, throw_stick};

let mut rng = StdRng::seed_from_u64(20261024);
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let knocked_over = throw_stick(game.stick(), game.field().skittles(), &mut rng);
let point = calc_score(&knocked_over);
let next_field = relocate_skittles(game.field(), &mut rng);
game = game.record_point(1, point).with_field(next_field);
assert_eq!(game.teams()[1].score_board().history(), &[point]);
assert_eq!(game.field().skittles().len(), 12);
```

- 各ピンの倒れる確率は`1 / (1 + 距離)`です。遠いほど倒れにくくなります。
- 判定後、倒れなかったピンも含め全ピンを隣接8マスのいずれかへ移動します。整数座標の端でも範囲内へ移動します。
- `Field::default()`は1〜12番を3列4行（x=-1,0,1、y=3,4,5,6）へ配置します。空・任意配置も指定できます。
- 同じ座標への配置は許容します。衝突計算はありません。`Skittle::with_position`は元のピンを変えず新しい値を返します。
- 次回の投てきには返された新しいFieldを使います。同じ初期配置とシードで再現でき、randのバージョン間の同一結果は保証しません。

すべてのドメイン型（Position・Skittle・Stick・Point・ScoreBoard・Team・Molkky・Field）は、非公開フィールドと共有参照のAPIを持ち、既存オブジェクトを書き換えません。
更新結果は`let updated = original.add(point)`や`game = game.record_point(index, point)`のように受け取ります。
元の値は引き続き参照でき、更新結果を捨てると`must_use`警告が出ます。
`fall_probability`はStickとSkittleからその都度計算し、Skittleに確率の状態は保持しません。移動と投てきの乱数源は呼び出し側が渡します。
TUIは返された最新の値を保持し直します。乱数源と端末I/Oには可変状態があり、プログラム全体が純粋関数だけで構成されているわけではありません。

以前の`Throw`・`Score`・`with_throw`は廃止しました。倒れたピンは`Vec<Skittle>`、採点結果は`Point`、履歴は`ScoreBoard`へ分けています。
`Molkky::record_throw`は`record_point`へ、`Team::score`は`score_board`へ変更しました。Teamの得点表の差し替えは`with_score_board`を使います。
APIの使用例と型による制約は`tests/api.md`を通じたドキュメントテストで検証します。
レビュー・設計・テスト計画は[rust_molkky/DEVELOPMENT.md](rust_molkky/DEVELOPMENT.md)に記録しています。

## Java演習環境

- OpenJDK 17（17.0.19以降）が必要です（JREのみではコンパイルできません）。
- 必要に応じて、`JAVA_HOME`をJDK 17のインストール先に設定してください。
- GradleはWrapper経由で取得するため、事前インストールは不要です。
- 初回実行時は、Gradle・JUnit・静的検査ツールの取得にインターネット接続が必要です。
- 実装は`java_molkky/src/main/java`、JUnit 5のテストは`java_molkky/src/test/java`へ配置します。

```sh
cd java_molkky
./gradlew test        # テストを実行
./gradlew build       # テストとビルドを実行
./gradlew clean       # ビルド結果を削除
```

Windowsでは`./gradlew`の代わりに`gradlew.bat`を使用します。
`java_molkky/src/test/java/molkky/ThrowTest.java`に、モルックのテストを配置しています。
`java_molkky`内で`./gradlew test --tests molkky.ThrowTest`を実行すると、このクラスのみ実行できます。

Java版の`Throw`には、番号を直接渡す代わりに`Skittle`を渡します。

```java
new Throw(new Skittle(5), new Skittle(8)).calcPoint(); // 2
new Throw().calcPoint();                             // 0
```

`Skittle`は番号1〜12を保証し、番号の型が`int`なのでnullは渡せません。
`Throw`は生成時に重複番号・null配列・nullのSkittleを拒否します。

STEP 02の現在の得点は、投てき列を`Score`に渡して求めます。

```java
Throw first = new Throw(new Skittle(1), new Skittle(2), new Skittle(3),
        new Skittle(4), new Skittle(5));
Throw second = new Throw(new Skittle(11));

new Score().value();              // 0
new Score(first).value();         // 5
new Score(first, second).value(); // 16
```

`Score`は生成時点の合計を保持する不変のオブジェクトです。
レビュー・設計・テスト計画と実行結果は[java_molkky/DEVELOPMENT.md](java_molkky/DEVELOPMENT.md)に記録しています。

Zedでリポジトリ全体を開く場合は、`.zed/tasks.json`がテストの実行先を`java_molkky`に設定します。
テスト横の▶から実行してください。古いタスクの再実行では、以前のコマンドが使われる場合があります。

### NullAwayによるnull検査

`./gradlew compileJava`・`./gradlew test`・`./gradlew build`のコンパイル時にNullAwayが動作します。
`molkky/package-info.java`の`@NullMarked`により、通常のコードとテストコードを非nullとして検査します。
新しいパッケージを作る場合も、`package-info.java`に`@NullMarked`を付けてください。

例えば、次の呼び出しは`[NullAway]`のコンパイルエラーになります。

```java
new Throw((Skittle) null);
new Score((Throw[]) null);
```

実行時の拒否を確認する異常系テストだけは、Nestedクラスの`@SuppressWarnings("NullAway")`で検査を抑制します。
テストの実行自体は省略しません。

静的検査で検出できないケースもあります。例えば、`Skittle[] pins = {new Skittle(1), null}; new Throw(pins);`は今回の検査を通ります。
このため、Throw・Scoreの実行時のnullチェックも維持しています。

JDK 17との互換性のためError Proneは2.42.0に固定しています。
JSpecifyモードに必要なOpenJDKの条件とコンパイラ設定は、[NullAway公式資料](https://github.com/uber/NullAway/wiki/JSpecify-Support#supported-jdk-versions)を参照してください。
Oracle JDK 17は、この設定の対象外です。

### JDKを導入しても認識されない場合

`JAVA_HOME`の設定だけでは、JREにコンパイラは追加されません。
`$JAVA_HOME/bin/javac -version`でJDK 17があることを確認してください。
JDKの導入後も同じエラーが続く場合は、以前の検出結果を保持しているGradleデーモンを停止します。

```sh
cd java_molkky  # リポジトリ直下にいる場合
./gradlew --stop
./gradlew javaToolchains   # Java 17の「Is JDK: true」を確認
./gradlew test
```
