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
| src/core/models/ | Skittle・SkittleState・Field・Stick・Point・ScoreBoard・Team・Id・Name・ScoreState |
| src/core/services/ | Molkkyの既定ルール（投てき・採点・Won/手番のチーム選択） |
| src/simulation/ | ArrayField、標準Field生成 |
| src/game/ | coreとviewの合成、表示上の移動、アプリの終了条件 |
| src/tui/ | 端末入力、得点表の更新と手番設定、描画の呼び出し |
| src/view/ | 表示専用のPosition、描画構造体、Widget・CanvasDraw |

core/modelsという名前は、エンティティと値オブジェクトの両方をまとめるためのものです。
Teamは参加チーム、PointやScoreBoardは不変な値として扱います。Molkkyは状態を所有する構造体ではなく、チーム一覧から採点・試合判定を行うルールのtraitです。
モデルの更新は新しい値を返し、乱数源と端末入出力は呼び出し側で扱います。
viewはcoreを参照せず、gameが渡す番号・得点・確率等の表示値から描画します。

### ライブラリの利用

```rust
use rust_molkky::core::models::{Name, Skittle, Team};
use rust_molkky::core::services::{Molkky, SimpleMolkky};

let rules = SimpleMolkky;
let original = vec![Team::new(Name::new("赤").unwrap()), Team::new(Name::new("青").unwrap())];
let board = original[0].score_board().add(rules.calc_score(&[Skittle::new(11)]));
let updated = original[0].with_score_board(board);
assert_eq!(original[0].score_board().value(), 0);
assert_eq!(updated[0].score_board().value(), 11);
assert!(rules.judge_game(&[updated]).is_none());

```

MolkkyとSimpleMolkkyはcrate直下・core直下・core/servicesから公開します。
calc_score・judge_game・next_teamはMolkkyのメソッドです。record_pointは廃止し、得点表の更新は利用側が行います。
PositionとSkittle/Stickの位置APIはcoreから削除しました。Positionはview内の表示専用型です。
coreのFieldはピン集合を参照するtraitです。位置や世界状態は具体的なField実装が持てます。
TUIのControllerはチーム一覧と配列Field・確率設定Stickを個別に保持します。gameがピンのIdと表示座標を対応づけます。

### チームの識別子と名前

Team::new(Name::new("赤").unwrap())で検証済みの名前からチームを作成します。
IdはUUID v4で生成し、同名でも別チームは別IDです。cloneや得点更新では同じIDを保持します。
名前は前後の空白を除去した後に1〜30文字とし、入力に制御文字が含まれる場合は拒否します。
文字数はRustのcharsによるUnicodeスカラー値の数です（表示幅や書記素クラスタの数ではありません）。
Name::newはNameErrorを返し、TUIでは不正な名前を端末初期化前に入力エラーとして報告します。
team.id()で一覧内の順番や名前に依存せず検索できます。同じIDの二重登録はjudge_gameが拒否します。
Teamはturn: boolを保持し、turn()/with_turn(bool)で参照・不変更新します。初期値はfalseです。
手番の初期設定・循環・失格者のスキップ・終了時の解除は使用側が行います。

### Field生成・投てき・採点

FieldとStickは、非公開のIdだけを持つ不変の型です。new()/default()は新しいUUIDを発行し、id()で取得できます。
simulationのArrayFieldがFieldと個体一覧、RandomStickがStickと検証済み確率を合成します。
Molkky::hit(&Stick, &[Skittle], JudgeStrategy)は判定関数を呼び、更新後の全個体を返します。
JudgeStrategy<'a>は&'a mut dyn FnMut(&Stick, &[Skittle]) -> Vec<Skittle>の型別名です。専用traitやヒープ確保は不要です。
判定関数は乱数源や物理世界を捕捉できます。coreは乱数・座標・描画に依存しません。

```rust
use rust_molkky::{SkittleState, Stick, Molkky, SimpleMolkky};
use rust_molkky::simulation::{RandomStick, StandardFieldGenerator};
let field = StandardFieldGenerator.generate();
let stick = RandomStick::new(Stick::new(), 1.0);
let mut rng = rand::rng();
let updated = SimpleMolkky.hit(stick.stick(), field.skittles(), &mut |_, pins| stick.judge(pins, &mut rng));
let fallen: Vec<_> = updated.into_iter().filter(|pin| pin.state() == SkittleState::Fall).collect();
assert_eq!(SimpleMolkky.calc_score(&fallen).value(), 12);
```

StandardFieldGeneratorは1〜12を各1本生成し、ArrayField::new(field, skittles)は任意の個体集合を保持します。
Field/FieldGeneratorのtraitは廃止しました。座標付きの独自表現はcoreのFieldと合成できます。
SkittleはId・番号・状態（Stand/Fall）を持ち、同じ12番を2本置けます。newはStand、with_stateは同じIDの新しい値を返します。
hitは入力と結果のID重複・個体集合の変更を拒否します。判定方法は状態だけを変更し、ID・番号を維持します。
calc_scoreは渡された採点対象だけを扱います。全個体をそのまま採点せず、今回StandからFallへ変化した個体を呼び出し側で選びます。
RandomStickの既定確率は0.25で、0〜1以外・NaN・無限大は拒否します。既にFallの個体は再抽選しません。
この確率方式はゲーム用の簡略モデルです。物理エンジンや表示用の型引数は追加していません。

0本なら0点、1本なら番号、複数本なら本数です。13本以上の変則構成も本数で採点します。
ScoreBoard::new(history)は生成時に履歴を検証し、Calculated・Reset・Disqualified・Wonの状態を保持します。
ScoreBoard::default()は空履歴・0点・Calculatedです。addは履歴を追加した新しい得点表を同じ生成経路で作ります。
公開する問い合わせはhistory()（共有スライス）、total()（正規化済み履歴の合計）、state()（状態）です。
50点超過で25点となりReset、50点ちょうどでWon、3回連続0点でDisqualifiedになります。
超過時は履歴を[25]へ置き換え、その後の得点を追記します。25は補正後の基準値であり、1投の得点ではありません。
history()は直近の補正以降の計算用履歴です。投数や全投てきの記録は呼び出し側で管理します。
Resetは次の通常投てきでCalculatedへ戻ります。Won/Disqualifiedに達した後の追加履歴は生成・追加時に拒否します。
履歴だけからnewで生成し直す場合、[25]だけでは直前の補正を復元できずCalculatedとなります。得点・連続0点・勝利/失格の判定は維持します。
is_resetは内部の超過判定です。apply_point・was_resetは削除しました。

### 採点・試合ルールのデフォルト実装

core/services/molkky.rsのMolkky traitへ、validate_unique_ids・hit・calc_score・judge_game・next_teamを直接実装しています。
SimpleMolkkyはその既定メソッドをそのまま使う、状態を持たない型です。
得点履歴・合計・25点補正はScoreBoardが担当し、同じ処理をtraitへ複製していません。
next_teamは手番フラグの検索だけを行い、順番の計算・失格スキップ・終了判定を行いません。
TUIはSimpleMolkkyを保持し、その参照を表示用合成にも渡します。coreには端末入出力やゲームの操作ループを置きません。
Field生成・描画はこのtraitには含めません。

### チーム選択と使用側の進行

Molkky::judge_game(&teams)はScoreState::Wonのチームを共有参照で返します。WonがなければNoneです。
Molkky::next_team(&teams)はturn=trueのチームを共有参照で返します。フラグがなければNoneです。
複数勝者・チームIDの重複・複数手番は拒否します。judge_gameは空の参加者も拒否します。
next_teamは失格や終了を見て手番を推測・変更しません。

利用側はScoreBoard::addで得点表を作り、Team::with_score_boardでチームを更新します。
TUI側で終了後の投てきを停止し、投てき後に次の失格していないチームへwith_turnで手番を設定します。
終了時は全チームの手番を解除します。activeインデックスはControllerに重複保存しません。
TUIの投数はチームごとに保持し、得点履歴が[25]へ圧縮されても投てき番号と投数表示を維持します。

既存アプリの「複数参加から最後の生存者が勝つ」ルールはgame/progress.rsのwinning_teamで扱います。
coreのWon検索と組み合わせますが、得点0の生存者のScoreStateをWonへ書き換えません。
単独参加・全失格・50点到達の振る舞いは維持します。

### 表示とテスト

初期表示は投てき側から1・2、3・10・4、5・11・12・6、7・9・8の4段です。座標の数値は描画用で、距離の単位ではありません。
今回倒れたピンだけを表示上で隣接位置へ移し、その場所でStandへ戻します。Field・Stick・SkittleのIDは変えず、共通の表示を次のチームも使います。

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
