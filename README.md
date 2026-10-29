# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

rust_molkkyは、標準入出力でチームの得点を記録するCUIと、ドメインモデルのライブラリです。
バイナリのmainから`cui::run::run`へ標準入出力を渡します。

```sh
cd rust_molkky
cargo run --locked
# 名前を引数で指定する場合
cargo run --locked -- 赤チーム 青チーム
# すべてのRustテスト
cargo test --locked
# 対象を絞る場合
cargo test --locked --test core_models
cargo test --locked --test application
```

| 場所 | 責務 |
| --- | --- |
| src/core/models/ | Team・Skittle・ScoreBoardなどのドメインモデル |
| src/core/services/molkky.rs | aim・hitの抽象宣言、採点・勝者・次のチームの判定 |
| src/cui/models/teams.rs | Vec<Team>の保持、CUI用の名前検証・Team生成 |
| src/cui/models/field.rs | 1〜12番のVec<Skittle>の保持 |
| src/cui/services/molkky.rs | CuiMolkky：入力番号による個体の選択と転倒結果の生成 |
| src/cui/io/input.rs | Input：入力元の保持、読み取り・変換・検証・再入力 |
| src/cui/io/output.rs | Output：出力先の保持、プロンプト・得点表・結果の表示 |
| src/cui/run.rs | チーム設定、投てき・採点・交代・終了のゲームループ |

InputとOutputはプレゼンテーション層の入出力処理なので、ドメインサービスの実装とは別の`io/`へ配置します。
`Teams`と`Field`は`cui::models`、`CuiMolkky`は`cui::services`、`Input`と`Output`は`cui::io`から公開します。

### CUIの操作

引数なしでは、チーム数（1以上）と各チーム名を入力します。名前は前後の空白を除いて1〜30文字、制御文字は使用できません。同名でも別のチームとして扱い、入力順に投げます。

- 倒れた番号を空白区切りで入力します。`5`は5点、`5 8`は2点です。番号は1〜12、同じ番号は1回だけ入力できます。
- `0`は倒れた本がない投てきです。
- `q`またはEOFで終了します。不正な入力は投数と手番を進めず再入力します。
- 入出力エラーは呼び出し元へ返し、プロンプトは入力待ちの前にflushします。

CUIは入力された転倒結果を記録する方式です。乱数や命中率は使いません。フォルト・記録訂正・各チーム12投制限は含めません。

### aimからhitへ渡す

```rust
use rust_molkky::{Molkky, Stick};
use rust_molkky::cui::{models::Field, services::CuiMolkky};

let field = Field::default();
let stick = Stick::new();
let rules = CuiMolkky::new(vec![5, 8]);
let targets = rules.aim(&stick, &field.skittles);
let fallen = rules.hit(&stick, &targets);
assert_eq!(rules.calc_score(&fallen).value(), 2);
assert_eq!(field.skittles.len(), 12);
```

`CuiMolkky`はInputで検証済みの番号を受け取ります。aimは入力順にField内の個体を選び、hitは選択された本を転倒状態にした結果を返します。元のFieldと個体IDは保ち、投てきごとにSkittleを新規生成しません。

`Molkky`を実装する型はaimとhitを定義します。calc_score・judge_game・next_teamにはデフォルト実装があります。得点表や手番の更新はrunがモデルのメソッドを使って行います。

### テストと残課題

テスト本体は対象ファイルに合わせて`tests/core/models/`、`tests/core/services/`、`tests/cui/models/`、`tests/cui/services/`、`tests/cui/io/`、`tests/cui/run.rs`へ配置します。`tests/core_models.rs`と`tests/application.rs`がモジュールを登録します。

ScoreBoardは投てき履歴を保持して順に再計算します。50点を超えたら25点、3回連続のミスなら失格・0点となり、勝利・失格の後は追加記録を拒否します。単独参加では失格または50点まで継続します。
不戦勝の判定はjudge_gameが行い、runが勝者の得点表へwith_winで50点・勝利を反映します。with_win自体は他チームの状態を調べず、元の履歴を変更しません。

Rustの全124件が成功しています。実行結果と変更理由は[DEVELOPMENT.md](DEVELOPMENT.md)を参照してください。

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
