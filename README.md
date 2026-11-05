# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

rust_molkkyは、標準入出力で遊ぶCUI、DioxusとRapier2Dによる見下ろし型GUI、共通のcoreライブラリです。
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
| src/core/services/molkky.rs | 照準・命中・連鎖の契約、採点・勝者・次のチームの判定 |
| src/core/models/stick.rs | Stickの状態、Stick→StickのThrow/Moveの契約 |
| src/core/models/skittle.rs | Skittleの状態、個体自身を立て直すStandUpの契約 |
| src/cui/models/ | coreモデルを内包するCuiXxxxの委譲型（AsRefで参照） |
| src/cui/services/teams.rs | 名前の検証・Vec<Team>の生成と追加 |
| src/cui/models/field.rs | CuiField：core::Fieldと1〜12番のVec<Skittle>の保持 |
| src/cui/models/stick.rs | CuiStick：既存のStickと転倒確率の保持 |
| src/cui/services/molkky.rs | CuiMolkky：入力番号による個体の選択、棒の確率の取得と転倒判定 |
| src/cui/io/input.rs | Input：入力元の保持、読み取り・変換・検証・再入力 |
| src/cui/io/output.rs | Output：出力先の保持、プロンプト・得点表・結果の表示 |
| src/cui/run.rs | チーム設定、投てき・採点・交代・終了のゲームループ |
| src/gui_2d/views/app.rs | モデル・共有ワールドの保持、採点・勝敗・交代の接続、画面全体の合成 |
| src/gui_2d/models/ | GuiField・GuiStick・GuiSkittle・GuiTeam・GuiScoreBoardのcore委譲型 |
| src/gui_2d/models/field.rs | 渡されたcore::Fieldの識別子と固定範囲の保持 |
| src/gui_2d/services/field.rs | フィールド・棒・標準12本の生成と、モデルからの物理ワールド構築 |
| src/gui_2d/services/teams.rs | 名前からチームを生成 |
| src/gui_2d/views/stick.rs | Stickコンポーネント：棒・照準の描画、マウス入力・蓄力・解放 |
| src/gui_2d/views/field.rs | Fieldコンポーネント：共有ワールドの固定刻み更新、hit/chain_hitの接続、照準の反映、一投の完了通知 |
| src/gui_2d/views/field.js | 同期pointer capture・取消・フレーム時刻通知と、破棄時の後始末 |
| src/gui_2d/services/molkky.rs | 接触・停止結果に対するaim/hit/chain_hit、coreルールの利用 |
| src/gui_2d/views/ | AppコンポーネントとSVG・速度メーター・得点表・結果の描画部品 |
| src/bin/gui_2d.rs | Webまたはdesktopレンダラーの起動 |

InputとOutputはプレゼンテーション層の入出力処理なので、ドメインサービスの実装とは別の`io/`へ配置します。
`cui::models`は`CuiField`・`CuiStick`・`CuiSkittle`・`CuiTeam`・`CuiScoreBoard`を公開します。単純な委譲型は`From<Core>`で生成して`AsRef<Core>`で参照します。Id・Name・Pointなどの値オブジェクトと状態enumにはcoreの型を直接使います。`CuiField`と`CuiStick`はCUI固有の情報を合成します。チーム一覧には`Vec<Team>`を使い、`cui::services::teams::{from_names, add_name}`が名前の検証・生成を担当します。`CuiMolkky`は`cui::services`、`Input`と`Output`は`cui::io`から公開します。

### CUIの操作

引数なしでは、チーム数（1以上）と各チーム名を入力します。名前は前後の空白を除いて1〜30文字、制御文字は使用できません。同名でも別のチームとして扱い、入力順に投げます。

- 狙う番号を空白区切りで入力します。番号は1〜12、同じ番号は1回だけ入力できます。
- 通常は狙った各スキットルが独立に50%で倒れます。例えば`5 8`を狙うと、倒れた本に応じて0点・5点・8点・2点のいずれかになります。
- `0`は倒れた本がない投てきです。
- `q`またはEOFで終了します。不正な入力は投数と手番を進めず再入力します。
- 入出力エラーは呼び出し元へ返し、プロンプトは入力待ちの前にflushします。

転倒判定には既存のrandクレートを使います。50%はこのCUIの既定値です。フォルト・記録訂正・各チーム12投制限は含めません。

### aimからhitへ渡す

```rust
use rust_molkky::{Molkky, SkittleState, Stick};
use rust_molkky::cui::{models::{CuiField, CuiStick}, services::CuiMolkky};

let field = CuiField::default();
let stick = CuiStick::new(Stick::new(), 0.8);
let rules = CuiMolkky::new(vec![5, 8]);
let targets = rules.aim(&stick, &field.skittles);
let fallen: Vec<_> = targets.iter()
    .map(|skittle| rules.hit(&stick, skittle))
    .filter(|skittle| skittle.as_ref().state() == SkittleState::Fall)
    .map(|skittle| *skittle.as_ref())
    .collect();
println!("{}点", rules.calc_score(&fallen).value());
assert_eq!(field.skittles.len(), 12);
```

`CuiMolkky`はInputで検証済みの番号を受け取ります。aimは入力順にField内の個体を選びます。CUIの実行ループはthrowを呼ばず、設定済みの確率でシミュレーションします。hitは1本ずつ確率判定し、更新後のCuiSkittleだけを返します。転倒時には投げたStickの命中確率を連鎖用に保持します。結果は内側のSkittleStateで判定し、既にFallの個体は状態と確率を保ちます。呼び出し側が今回倒れた個体をまとめて採点します。元のFieldと個体IDは保ち、投てきごとにSkittleを新規生成しません。

`Molkky`を実装する型は関連型Stick・Skittleとaim・hit・chain_hitを定義します。CuiMolkkyのStickはCuiStick、SkittleはCuiSkittleです。aim・hit・chain_hitは関連型の個体を返し、CUI固有の確率を保持します。calc_score・judge_game・next_teamにはデフォルト実装があります。得点表や手番の更新はrunがモデルのメソッドを使って行います。runは第4引数に`&CuiStick`を受け取り、起動元のmainが既定の50%の棒を渡します。

ThrowとMoveはcore/models/stick.rsに置く個体の操作契約です。`throw(stick: Self) -> Self`と`r#move(stick: Self) -> Self`はStickだけを値で受け、同じIdを保った新しいStickを返します。moveはRustの予約語なのでr#moveと表記します。状態はcore::StickStateのHeld/Move/Stopを使い、GUIに同じ状態のenumや完了boolは作りません。

GuiStick::newは保持位置・零実速度を初期化し、目標と強さを設定します。GUIでは解放時のGuiStick::r#move(stick)がその値から初速度ベクトルを生成し、HeldからMoveへ変更します。その後は物理結果をGuiStick::updateで反映し、moveが実速度と角速度からMove/Stopを返します。GUIのthrow実装はありません。CuiStick::throw(stick)は0以上1未満の確率を生成しますが、CUIのシミュレーションでは使いません。既定の50%や呼び出し側が指定した確率はそのまま維持します。

chain_hit(fallen_skittle, chained_skittle)は、呼び出し側が指定した対象へ連鎖を判定します。初回転倒が80%なら次は60%、さらに次は40%と20パーセントポイントずつ下げ、成功した対象へ減衰後の確率を引き継ぎます。0%より下には下げません。連鎖元がStand、対象が既にFall、判定失敗の場合は対象をそのまま返します。現在のCUI入力ループは指定番号への直接hitを行い、隣接探索や連鎖対象の自動選択は追加していません。

`StandUp`はcore/models/skittle.rsの抽象トレイトです。`skittle.stand_up()`が同じId・番号の新しいStandモデルを返します。CUIでは連鎖確率を0へ、GUIでは移動後の位置を保って角度と照準を初期化します。元のモデルは変更しません。GUIのAppは立て直したモデルから共有ワールドを作り直し、速度・角速度・接触履歴をクリアします。

### テストと残課題

自動テストはcore・cui・gui_2dの`models/`・`services/`に限定し、対象と同名のファイルを`tests/`以下に置きます。入力・出力・run・views・DOM操作のテストは保持しません。CUI/GUIのMolkky具象テストはソースのcfg(test)から同名のテストファイルを登録し、乱数源や実エンジンの結果からサービスの契約を確認します。

ScoreBoardは投てき履歴を保持して順に再計算します。50点を超えたら25点、3回連続のミスなら失格・0点となり、勝利・失格の後は追加記録を拒否します。単独参加では失格または50点まで継続します。
不戦勝の判定はjudge_gameが行い、runが勝者の得点表へwith_winで50点・勝利を反映します。with_win自体は他チームの状態を調べず、元の履歴を変更しません。

GUIを含むモデル・サービスのテストでは、hitの状態保持・連鎖・採点・失格・勝敗・立て直しを確認しています。確率0%・100%で転倒結果を固定し、得点・履歴・終了の既存の期待値も検証しています。実行結果と変更理由は[DEVELOPMENT.md](DEVELOPMENT.md)を参照してください。

### GUI 2Dの起動

赤チーム・青チームで開始します。フィールド内で左ボタンを押し、マウスで照準を合わせて離すと投擲します。領域外での解放も処理し、focus喪失時は蓄力を取り消します。停止後に採点・交代し、勝者が決まると結果を表示します。

Dioxus CLI（`dx`）が導入済みなら、次のコマンドでWeb版を起動できます。`dx`が対応するwasm-bindgenなどのビルドツールを用意します。

```sh
cd rust_molkky
rustup target add wasm32-unknown-unknown
dx serve --web --bin gui_2d --no-default-features --features gui_2d_web --locked --addr 127.0.0.1 --port 8080
# http://127.0.0.1:8080 を開く。Ctrl+Cで終了。
```

`dx`を使わず既存の起動スクリプトを使う場合は、Python 3と、Cargo.lockに対応するwasm-bindgen-cli 0.2.129を用意します。

```sh
cd rust_molkky
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/gui_2d_web.sh
# http://127.0.0.1:8080 を開く。Ctrl+Cで終了。
```

デスクトップ版は次のコマンドで起動します。LinuxではGTK/WebKit等の開発ライブラリが必要です。Debian/Ubuntu系では、pkg-config、libgtk-3-dev、libwebkit2gtk-4.1-dev、libxdo-dev、libayatana-appindicator3-dev、libssl-devとCコンパイラを用意してください。

```sh
cd rust_molkky
cargo run --locked --no-default-features --features gui_2d_desktop --bin gui_2d
```

CUIは既定のcui feature、GUIはgui_2d_webまたはgui_2d_desktopを選択します。Webの起動スクリプトはCUI依存を除外し、UUIDのjs機能を有効にします。単なるgui_2d featureはヘッドレスのテスト・ライブラリ利用向けで、起動時はレンダラーを選択してください。

`views::App`がチーム・GuiField・GuiStick・Vec<GuiSkittle>と、唯一のGuiMolkkyを保持します。`services::field::create_field`が標準配置のモデルを、create_worldがそれに対応する物理ワールドを用意します。Stickは入力、Fieldは一つのフレームループから物理更新と完了通知を担当し、Appがcoreの採点・勝敗・交代を接続します。Gameサービスはありません。全チームで同じフィールドと棒を共有します。操作用の初期投擲距離は1m、高さ方向の転倒は物理サービスの力積による近似です。

表示範囲はx=-0.3〜0.3m、y=-0.15〜1.35mで固定し、物体の移動による拡大縮小は行いません。`GuiField::BOUNDS`と`WALL_THICKNESS`を描画と物理で共用し、外周の厚さ2cmの壁で棒・スキットルが反射します。反発係数は0.5で、既存の減衰と合わせて停止します。壁と反射はこのGUIの取り決めです。

### GUIの検証

```sh
cd rust_molkky
cargo test --locked --features gui_2d
cargo test --locked --no-default-features --features gui_2d
cargo clippy --locked --all-targets --features gui_2d -- -D warnings
# サービスの契約を絞って確認する場合
cargo test --locked --features gui_2d --lib gui_2d::services
cargo test --locked --features gui_2d --test gui_2d
```

入力と表示はWeb・デスクトップ上で操作して確認します。今回、WebはChromiumで確認し、デスクトップはコンパイルのみ確認しています。

### GUI 2Dの描画部品

`gui_2d` featureを有効にすると、`gui_2d::views`から`App`・`Field`・`Skittle`・`Stick`・`Speedometer`・`Scores`・`Result`コンポーネントを利用できます。

Fieldは共通フィールドをSVGで上から描き、Appから渡されたモデルのSignalをStickへ渡します。Stickは棒と照準を、Skittleは各GuiSkittleの位置・状態・照準強調を描きます。寸法にはモデルの関連定数を使います。ScoresとTeamScoreは同じチームのSignalを参照し、Resultは判定済みの勝者名を受け取ります。FieldSurfaceは共通SVG、StickShapeとAimは棒と照準の描画を担当します。

```sh
cd rust_molkky
cargo test --locked --features gui_2d --test gui_2d
cargo build --locked --features gui_2d
```

フィールド・棒・スキットルは共通SVGのviewBox内へ配置します。SVG自体の縦横比を2:5へ揃え、全体を等比拡縮します。物理座標や個々の寸法は変えません。既存CUIの起動方法は変わりません。

### GUI 2Dの投擲サービス

`gui_2d::services::GuiMolkky`は共有のRapier2Dワールドを所有します。services::field::create_worldはGuiStickとGuiSkittleのモデル一覧を受け、Idで対応するボディを生成します。aimは引数の照準条件、hitは共有ワールドの接触結果を使い、StickのIdによる拒否は行いません。座標・寸法はm、速度はm/s、角度はラジアンです。coreに座標・Rapier型は追加しません。

| API | 動作 |
| --- | --- |
| aim | Rapierの形状問い合わせで候補を選び、位置・角度・照準状態を反映したGuiSkittleを返す |
| hit | 指定したStickとSkittleの接触とボディ状態から、位置・角度・状態を更新したGuiSkittleを返す |
| chain_hit | 指定した2個体の実際の接触ペアから、対象の物理状態を反映したGuiSkittleを返す |

FieldがRapierの共有ワールドを固定刻みで一度ずつ進めます。独自のstep APIとphysics.rsはありません。実際の接触履歴から、棒とピンにはhit、ピン同士には双方へchain_hitを適用します。接触履歴のないHitで再開したモデルもhitで停止まで追跡します。状態の判定はMolkkyの具象実装へ集約します。モデルのupdateはIdを保ち、元の値を変更せず新しいモデルを返します。

Stick自身が止まるとStopになります。スキットルは移動中Hit、停止時に接触力積からFallまたはStandへ確定します。FieldはMoveまたはHitがある間更新を続け、「StickがStop、かつHitが0本」でAppへ完了を通知します。AppはFallだけをcalc_scoreへ渡し、ScoreBoard.addとTeam.with_score_boardを反映します。弱い衝突はStandへ戻り、得点には含めません。継続時は各GuiSkittleのstand_upで立て直し、create_worldで同じ配置とIdのボディを作り直します。

半径・質量はGuiSkittle、棒の半径・長さ・質量はGuiStickの関連定数です。時間刻み・共通減衰はワールド生成を行うservices/field.rs、転倒閾値はmolkky.rsに置きます。時間刻みは従来どおりservices::TIME_STEPから参照できます。新しいワールドラッパー、Observer trait、イベントバス、開始用のラッパーは追加していません。

### GUI 2Dの入力と進行

StickはDioxusのonpointerdown/move/up/cancelを受け取り、一つのuse_coroutineで順番に処理します。内部のStickInputは所有ポインター・蓄力開始時刻・最後の有効目標だけを保持します。SVGの領域はイベントごとにMountedDataで取得します。縦横比がフィールドと一致するため、入力座標は領域内の割合とY反転だけで物理座標へ変換します。領域外では最後の有効目標を使います。

左押下で同じIdのHeldのGuiStickへ目標を設定します。FieldがMolkky::aimの結果を各Skittleへ反映します。押下中は時間ベースのspeed（0〜10m/s、往復2秒）を更新し、実速度は0、位置も保持します。解放時はその瞬間の目標とspeedでmoveを一度呼び、Moveへ進めます。領域外では最後の有効目標を使い、0速度・方向なしでは投擲しません。

Fieldの一つのループがRapierを固定刻みで更新し、棒へ運動結果を反映します。実際の接触履歴をhit/chain_hitへ渡し、スキットルの位置とHit/Fall/Standを取得します。Appは完了通知から採点・judge_game・next_teamを接続し、返却先へturnを反映します。最後に同じIdのStickをHeldへ戻すことで二重採点を防ぎます。進行用の別モデルや完了フラグは持ちません。

Ready・Charging・Simulatingは棒の状態・目標・Hitの有無から、終了はチームの手番から求めます。モデルの所有元はAppに揃え、各描画部品にcoreモデルを重複所有させません。

views/field.jsはDioxusの共通APIにない同期pointer capture、focus喪失時のpointercancel、requestAnimationFrameの時刻通知を補います。マウス入力と座標変換はRust側です。取消は蓄力中の照準・速度・入力状態を戻し、別pointerの取消や投擲中のfocus喪失では進行を戻しません。Fieldの破棄時はuse_dropでJSへ終了を通知し、JSは通常終了でも接続失敗でもfinallyでリスナー・フレーム・captureを解除します。Webとdesktopで同じコンポーネントを使います。

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
