# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

rust_molkkyは、モルックのドメインモデルとMolkkyサービスを公開するライブラリです。
`src/lib.rs`はcoreだけを公開します。CLI・TUI・描画・シミュレーションの実装は削除済みです。

```sh
cd rust_molkky
cargo check --locked --lib
cargo test --locked --test core
cargo test --locked --test models
```

サービステストは28件成功しています。モデルのテストには得点履歴・失格・終了後の記録に関する既存13件の失敗があります。
全体の`cargo test --locked`は、削除済みのCLI・simulationを参照するテストなどでコンパイルに失敗します。
今回の実行結果と残課題は[DEVELOPMENT.md](DEVELOPMENT.md)を参照してください。

| 場所 | 責務 |
| --- | --- |
| src/core/models/ | Skittle・Point・ScoreBoard・Team・Id・Nameなどのモデル |
| src/core/services/ | 投てき判定の呼び出し、採点、50点到達済みチームと次のチームの検索 |

### ライブラリの利用

```rust
use rust_molkky::{Molkky, Name, SimpleMolkky, Skittle, Team};
let teams = [
    Team::new(Name::new("赤")).with_turn(true),
    Team::new(Name::new("青")),
];
let point = SimpleMolkky.calc_score(&[Skittle::new(5)]);
let scored = teams[0].with_score_board(teams[0].score_board().add(point));
assert_eq!(scored.score_board().total(), 5);
assert_eq!(teams[0].score_board().total(), 0);
let next = SimpleMolkky.next_team(&teams).unwrap();
assert_eq!(next.id(), teams[1].id());
assert!(teams[0].turn());
assert!(SimpleMolkky.judge_game(&teams).is_none());
```

モデル・Molkky・SimpleMolkkyはcrate直下およびcoreから公開します。
Molkkyは状態を持たず、得点表や手番フラグの更新にはモデルの不変更新を使います。

### Molkkyの契約

- `calc_score(&[Skittle]) -> Point`：0本は0点、1本は番号、2本以上は本数です。今回倒れた個体の選別は呼び出し側で行います。
- `hit(&Stick, &[Skittle], JudgeStrategy<'_>) -> Vec<Skittle>`：注入した判定関数を1回呼び、所有する結果をそのまま返します。
- `judge_game(&[Team]) -> Option<&Team>`：ScoreState::Wonのチームを検索します。該当しなければNoneです。最後の生存者の得点を50点へ変更する試合進行は、今回の実装範囲に含めません。
- `next_team(&[Team]) -> Option<&Team>`：現在の手番の次から循環し、失格者を飛ばして選びます。未設定なら先頭の有効チーム、単独参加なら同じ有効チームを選びます。空・全失格・50点到達・複数参加で最後の一チームだけになった場合はNoneです。

next_teamは元の一覧や手番を変更しません。手番を更新するときは、返されたチームのIDを使って呼び出し側が設定します。
judge_gameとnext_teamの返す参照はチーム一覧から借用し、サービスの借用期間には依存しません。
JudgeStrategyは`&mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>`の型別名です。クロージャで乱数源などを捕捉でき、結果は入力の借用期間後も保持できます。

MolkkyにID重複・複数手番・複数勝者などの入力検証は置きません。入力側で一覧の整合性を確認する方針です。入力側の検証実装は未実装です。

### モデルとテスト

TeamはId・Name・ScoreBoard・手番を保持します。同名でも別チームは別IDになり、cloneや得点・手番の更新では同じIDを保ちます。
Name::new(&str)は文字列をそのまま保持するNameを返します。名前の検証・正規化は入力側の責務です。
Skittleは番号・状態・ID、FieldとStickはIDを保持します。位置や表示座標は扱いません。
得点表の計算はScoreBoardが担当し、Molkkyへ複製しません。

テスト本体は対象ファイル名に対応する`tests/core/models/`と`tests/core/services/`へ置き、modで階層化します。
`tests/core.rs`はサービスのテストを登録し、`tests/models.rs`はモデルの実ソースを組み込んで検証します。
API例は`tests/api.md`にあります。この文書には削除済みの構成の例も含まれ、現在の自動テストには登録されていません。

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
