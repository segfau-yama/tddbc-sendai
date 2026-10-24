# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

- `rust_molkky`は、Java版を起点に、チームとゲームの管理を追加したライブラリです。
- 実装は`rust_molkky/src/`に構造体ごとに分け、テストは`rust_molkky/tests/`にあります。
- STEP 01・02に加え、ユーザー指定の「50点ちょうどで終了、超過すると25点へ戻る」ルールを実装しています。
- Scoreは点数の生の履歴を保持し、投てきの順に超過ルールを適用して現在得点を求めます。
- `with_throw`は新しいScoreを返し、元の履歴は変更しません。`history()`は読み取り専用です。
- ランダム生成は外側の`random_throw`関数に分離し、randを使用しています。初回の依存取得にはインターネット接続が必要です。
- `Skittle::new`が番号1〜12を保証し、`Throw::new`が番号の重複を拒否します。
- `skittle.rs`のPositionで座標を表し、`Field`が番号と位置を持つスキットルを配置します。
- 倒れる確率は投てき位置との距離から求め、投てき後には全ピンが隣接マスへ移動します。
- 範囲外入力・重複の例外は、既存Rust版と同じく`panic!`で表現しています。
- Java版の可変長引数はスライスで表現します。通常のRustでは、値・参照・スライスにnullはありません。
- Optionを入力型に含めず、Noneを渡せないことを`tests/api.md`のドキュメントテストで検証しています。
- `src/lib.rs`は公開APIを再エクスポートし、`tests/api.md`をドキュメントとして読み込みます。
- テスト名は日本語とし、ドメインや状況ごとのmodにまとめています。

```sh
cd rust_molkky
cargo test           # テストを実行
cargo test --test throw_test  # Throwのテストだけ実行
cargo build          # ビルド
cargo fmt --check    # フォーマットを確認
cargo clippy --all-targets -- -D warnings  # 警告をエラーとして検査
```

| 構造体 | 実装 | テスト |
| --- | --- | --- |
| Skittle | `rust_molkky/src/skittle.rs` | `rust_molkky/tests/skittle_test.rs` |
| Throw | `rust_molkky/src/throw.rs` | `rust_molkky/tests/throw_test.rs` |
| Score | `rust_molkky/src/score.rs` | `rust_molkky/tests/score_test.rs` |
| Team | `rust_molkky/src/team.rs` | `rust_molkky/tests/team_test.rs` |
| Molkky | `rust_molkky/src/molkky.rs` | `rust_molkky/tests/molkky_test.rs` |
| Position | `rust_molkky/src/skittle.rs` | `rust_molkky/tests/position_test.rs` |
| Field | `rust_molkky/src/field.rs` | `rust_molkky/tests/field_test.rs` |

ランダム生成は`src/simulation.rs`、対応するテストは`tests/simulation_test.rs`にあります。

```rust
use rust_molkky::{Score, Skittle, Throw};

let first = Throw::new(&[
    Skittle::new(1), Skittle::new(2), Skittle::new(3),
    Skittle::new(4), Skittle::new(5),
]);
let second = Throw::new(&[Skittle::new(11)]);

assert_eq!(first.calc_point(), 5);
assert_eq!(Score::new(&[]).value(), 0);
assert_eq!(Score::new(&[&first, &second]).value(), 16);
assert_eq!(Score::new(&[&second, &second]).value(), 22);
```

ゲームへの記録は、対象チームのインデックスと確定したThrowを渡します。

```rust
use rust_molkky::{Molkky, Skittle, Team, Throw};

let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let twelve = Throw::new(&[Skittle::new(12)]);
for _ in 0..4 { game.record_throw(0, &twelve); }
game.record_throw(0, &Throw::new(&[Skittle::new(3)])); // 48 + 3で25点へ
assert_eq!(game.teams()[0].score().value(), 25);
assert_eq!(game.teams()[0].score().history(), &[12, 12, 12, 12, 3]);
assert_eq!(game.result(), None);

game.record_throw(0, &twelve); // 37点
game.record_throw(0, &twelve); // 49点
game.record_throw(0, &Throw::new(&[Skittle::new(1)])); // 50点
assert_eq!(game.result(), Some(0)); // 赤チームの勝利
```

- `record_throw`の戻り値は、その回の点数です。補正後の得点は`score().value()`で取得します。
- `result()`は進行中ならNone、終了時は勝者のインデックスをSomeで返します。
- 終了後の投てき・存在しないチームへの記録はpanicで拒否し、状態を変更しません。
- チームは1件以上必要です。空の名称・同名のチームは許容し、インデックスで識別します。
- 既に得点を持つTeamも渡せますが、複数の勝者がいる状態は拒否します。
- 手番の強制や、指定されていない失投・終了ルールは追加していません。

ランダムな投てきは、Field・投てき位置・乱数源を渡して生成します。

```rust
use rand::{rngs::StdRng, SeedableRng};
use rust_molkky::{random_throw, Field, Molkky, Position, Skittle, Team};

let mut rng = StdRng::seed_from_u64(20261024);
let mut game = Molkky::new(vec![Team::new("赤"), Team::new("青")]);
let mut field = Field::new(vec![
    Skittle::new(5).with_position(Position::new(0, 1)),
    Skittle::new(8).with_position(Position::new(0, 4)),
]);
let from = Position::new(0, 0);
let thrown = random_throw(&mut field, from, &mut rng);
let points = game.record_throw(1, &thrown);
assert_eq!(game.teams()[1].score().history(), &[points]);
// field.skittles()から投てき後の新しい配置を読み取れる。
```

- 座標は整数のグリッドです。`distance_to`はユークリッド距離を返します。
- 各ピンの確率は`1 / (1 + 距離)`です。同じ場所なら必ず倒れ、遠いほど倒れにくくなります。
- 倒れる判定は移動前の配置で行い、その後、倒れなかったピンも含めて全ピンが隣接8マスのいずれかへ移動します。
- i32の座標範囲の端では範囲内の隣接マスを選びます。同じ座標への配置は許容し、衝突計算は行いません。
- `Field::default()`は1〜12番を3列4行（x=-1,0,1・y=3,4,5,6）へ配置します。任意配置や空のFieldは`Field::new`で作れます。
- `Skittle::new(number)`は原点に生成します。座標を指定する場合は`with_position`を使います。
- 同じ番号のピンは、座標が違ってもThrow・Fieldの両方で重複として拒否します。

以前の`random_throw(&mut rng)`は、`random_throw(&mut field, from, &mut rng)`へ変更しています。
同じ初期配置とシードで再現確認を行っています。randのバージョン間での同一結果は保証しません。

`Throw::new`は整数スライスからSkittleのスライスへ変更しています。
`Score::new`はThrowの参照を受け取り、同じインスタンスも回数分加算します。
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
