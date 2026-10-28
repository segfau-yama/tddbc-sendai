# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

rust_molkkyは、モルックの採点・試合管理を行うライブラリとTUIです。
coreは表示座標を持たず、表示位置を変えても投てきの確率や採点は変わりません。

```sh
cd rust_molkky
cargo test --locked
cargo build --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- 赤チーム 青チーム
```

TUIは72列×24行以上の対話型端末で動作します。Space / Enterで投てき、矢印キーで棒の表示位置を移動、Rで再開、Q / Esc / Ctrl+Cで終了します。

| 場所 | 責務 |
| --- | --- |
| src/core/models/ | Skittle・Stick・Field・Point・ScoreBoard・Team・Id・Name・GameStatusというドメインモデル |
| src/core/services/ | calc_score（採点）、throw_stick（抽選）、judge_game（継続・勝敗判定）、next_team（次チーム）、record_point（記録制約と不変更新） |
| src/game/ | coreとviewの合成、投てき後の表示上の移動 |
| src/tui/ | Controllerによる端末入力、操作制御、描画の呼び出し |
| src/view/ | 表示専用のPosition、描画構造体、Widget・CanvasDraw |

core/modelsという名前は、エンティティと値オブジェクトの両方をまとめるためのものです。
Teamは参加チーム、PointやScoreBoardは不変な値として扱います。試合を包むMolkky構造体は廃止し、チーム一覧から状態を導出します。
モデルの更新は新しい値を返し、乱数源と端末入出力は呼び出し側で扱います。
viewはcoreを参照せず、gameが渡す番号・得点・確率等の表示値から描画します。

### ライブラリの利用

```rust
use rust_molkky::core::models::{GameStatus, Name, Skittle, Team};
use rust_molkky::core::services::{calc_score, judge_game, record_point};

let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let updated = record_point(&original, 0, calc_score(&[Skittle::new(11)]));
assert_eq!(original[0].score_board().value(), 0);
assert_eq!(updated[0].score_board().value(), 11);
assert_eq!(judge_game(&updated), GameStatus::InProgress);
```

従来のrust_molkky::Skittleやrust_molkky::core::calc_score等の公開名も再公開しています。
PositionとSkittle/Stickの位置APIはcoreから削除しました。Positionはview内の表示専用型です。
Fieldは全チーム共通のピン集合として残り、位置を持ちません。TUIのControllerがチーム一覧と共通のField・Stickを個別に保持します。

### チームの識別子と名前

Team::new(Name::new("赤").unwrap())で検証済みの名前からチームを作成します。
IdはUUID v4で生成し、同名でも別チームは別IDです。cloneや得点更新では同じIDを保持します。
名前は前後の空白を除去した後に1〜30文字とし、入力に制御文字が含まれる場合は拒否します。
文字数はRustのcharsによるUnicodeスカラー値の数です（表示幅や書記素クラスタの数ではありません）。
Name::newはNameErrorを返し、TUIでは不正な名前を端末初期化前に入力エラーとして報告します。
team.id()で一覧内の順番や名前に依存せず検索できます。同じIDの二重登録はjudge_gameが拒否します。
既存の手番・勝者・記録APIのindexは一覧内の位置として維持しています。

### 投てきと採点

Stick::new(probability)で倒れる確率を0〜1で指定します。既定値は0.25です。
throw_stickは各番号を独立して抽選します。これはゲーム用の簡略モデルで、公式の確率や物理シミュレーションではありません。
範囲外・NaN・無限大の確率、1〜12以外の番号、重複番号を拒否します。

```rust
use rust_molkky::{Skittle, Stick, calc_score, throw_stick};
let pins = [Skittle::new(5), Skittle::new(8)];
let knocked = throw_stick(&Stick::new(1.0), &pins, &mut rand::rng());
assert_eq!(calc_score(&knocked).value(), 2);
```

0本なら0点、1本なら番号、複数本なら本数です。ScoreBoard::addは履歴を追加した新しい得点表を返します。
50点超過で25点へ戻し、3回連続0点で失格とします。生の点数履歴は保持します。
ScoreBoard::is_winnerは既存APIとして50点到達を示し、ゲーム全体の勝者はjudge_gameが決めます。
was_resetは直近の25点補正の説明で引き続き使います。

### 勝敗判定

judge_game(&[Team])は失格チームを含む元の参加リストを受け取り、次を返します。

- GameStatus::InProgress：進行中。
- GameStatus::Finished { winner: Some(index) }：終了し、指定チームが勝利。50点到達、または2チーム以上の参加から最後の生存チームになった。
- GameStatus::Finished { winner: None }：全チーム失格で終了。

終了という共通状態に勝者の有無をまとめています。GameStatusは判定結果であり、チームや得点履歴そのものは所有しません。
単独参加は50点で勝ち、3回連続0点なら勝者なしで終了します。
空の参加者、複数の50点到達チーム、終了・失格後に記録がある不正履歴は拒否します。
record_point(&teams, index, point)は終了後・失格チーム・範囲外の記録を拒否し、新しいチーム一覧を返します。
next_team(&teams, current)は失格チームを飛ばした次のインデックスを返し、終了後はNoneを返します。
任意のチームへの記録は許容し、利用側がnext_teamの結果を手番として保持します。

### 表示とテスト

初期表示は投てき側から1・2、3・10・4、5・11・12・6、7・9・8の4段です。座標の数値は描画用で、距離の単位ではありません。
倒れたピンだけを表示上で隣接位置へ移し、立ったピンはそのままにします。共通の表示を次のチームも使います。

tests/core/models・tests/core/services・tests/game・tests/tui・tests/viewにテストを置き、対象ファイルと同じ名前にしています。
API利用例と型による制約はtests/api.mdのドキュメントテストで確認します。
レビュー・設計・Red/Green・検証結果は[rust_molkky/DEVELOPMENT.md](rust_molkky/DEVELOPMENT.md)に記録しています。

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
