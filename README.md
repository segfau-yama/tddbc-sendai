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
| src/core/services/molkky.rs | 照準・当たり判定・物理更新の契約、採点・勝者・次のチームの判定 |
| src/cui/models/ | coreモデルを内包するCuiXxxxの委譲型（AsRefで参照） |
| src/cui/services/teams.rs | 名前の検証・Vec<Team>の生成と追加 |
| src/cui/models/field.rs | CuiField：core::Fieldと1〜12番のVec<Skittle>の保持 |
| src/cui/models/stick.rs | CuiStick：既存のStickと転倒確率の保持 |
| src/cui/services/molkky.rs | CuiMolkky：入力番号による個体の選択、棒の確率の取得と転倒判定 |
| src/cui/io/input.rs | Input：入力元の保持、読み取り・変換・検証・再入力 |
| src/cui/io/output.rs | Output：出力先の保持、プロンプト・得点表・結果の表示 |
| src/cui/run.rs | チーム設定、投てき・採点・交代・終了のゲームループ |
| src/gui_2d/app.rs | Dioxusの起動・状態とDOM接続の寿命管理 |
| src/gui_2d/models/ | GuiField・GuiStick・GuiSkittle・GuiTeam・GuiScoreBoardのcore委譲型 |
| src/gui_2d/models/field.rs | 渡されたcore::Fieldと個体集合・固定範囲の保持 |
| src/gui_2d/services/field.rs | 標準12本・配置・棒・物理ワールドの組み立て |
| src/gui_2d/services/teams.rs | 名前からチームを生成 |
| src/gui_2d/controllers/mouse.rs | マウス入力の解釈・座標変換・蓄力・解放・取消 |
| src/gui_2d/controllers/app.rs | 進行・入力と物理の接続、状態に応じた画面・得点表・結果の合成 |
| src/gui_2d/controllers/field.rs | 描画用の位置の取り出し、フィールド内の描画部品の合成 |
| src/gui_2d/controllers/stick.rs・skittle.rs | 棒の待機/投擲中の表示、個体の位置と照準候補の表示制御 |
| src/gui_2d/app.js | DOMのpointer capture・イベントと時刻の通知 |
| src/gui_2d/services/molkky.rs | Rapierによる投擲・物理更新・転倒判定、coreルールの利用 |
| src/gui_2d/view/ | SVG、速度メーター、得点表、結果表示 |
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
use rust_molkky::{Molkky, SkittleState, Throw};
use rust_molkky::cui::{models::{CuiField, CuiStick}, services::CuiMolkky};

let field = CuiField::default();
let stick = CuiStick::default().throw(0.8);
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

`CuiMolkky`はInputで検証済みの番号を受け取ります。aimは入力順にField内の個体を選び、CuiStick::throw(probability)は確率（f64、0〜1）を更新したCuiStickを返します。hitは1本ずつ確率判定し、更新後のCuiSkittleだけを返します。転倒時には投げたStickの命中確率を連鎖用に保持します。結果は内側のSkittleStateで判定し、既にFallの個体は状態と確率を保ちます。呼び出し側が今回倒れた個体をまとめて採点します。元のFieldと個体IDは保ち、投てきごとにSkittleを新規生成しません。

`Molkky`を実装する型は関連型Stick・Skittleとfly・aim・hit・chain_hit・stand_upを定義します。CuiMolkkyのStickはCuiStick、SkittleはCuiSkittleです。aim・hit・chain_hitは関連型の個体を返し、CUI固有の確率を保持します。calc_score・judge_game・next_teamにはデフォルト実装があります。得点表や手番の更新はrunがモデルのメソッドを使って行います。runは第4引数に`&CuiStick`を受け取り、起動元のmainが既定の50%の棒を渡します。

Throw<Input>はcore/models/stick.rsに置くStick側の操作契約です。CuiStickはf64の確率、GuiStickはf32の速度(m/s)を受け、Idとその他の属性を保つ新しいSelfを返します。元のStickや物理ワールドは変更しません。値は呼び出し側が指定します。MolkkyのthrowとThrowOutputは削除しました。

chain_hit(fallen_skittle, chained_skittle)は、呼び出し側が指定した対象へ連鎖を判定します。初回転倒が80%なら次は60%、さらに次は40%と20パーセントポイントずつ下げ、成功した対象へ減衰後の確率を引き継ぎます。0%より下には下げません。連鎖元がStand、対象が既にFall、判定失敗の場合は対象をそのまま返します。現在のCUI入力ループは指定番号への直接hitを行い、隣接探索や連鎖対象の自動選択は追加していません。

flyは一投の動きを更新し、動いている間true、完了時はfalseを返す抽象メソッドです。CUIは飛行時間を持たずfalseを返し、Stickの確率を変えません。stand_upも抽象メソッドで、CUIの実装はId・番号を保ってStandと連鎖確率0へ戻した個体を返します。元の個体は変更しません。

### テストと残課題

テスト本体は対象ファイルに合わせて`tests/core/models/`、`tests/core/services/`、`tests/cui/models/`、`tests/cui/services/`、`tests/cui/io/`、`tests/cui/run.rs`へ配置します。`tests/core_models.rs`と`tests/application.rs`がモジュールを登録します。CUIのMolkky具象テストはソースのcfg(test)から同名のテストファイルを登録し、非公開の乱数源を固定して連鎖を検証します。

ScoreBoardは投てき履歴を保持して順に再計算します。50点を超えたら25点、3回連続のミスなら失格・0点となり、勝利・失格の後は追加記録を拒否します。単独参加では失格または50点まで継続します。
不戦勝の判定はjudge_gameが行い、runが勝者の得点表へwith_winで50点・勝利を反映します。with_win自体は他チームの状態を調べず、元の履歴を変更しません。

GUIを含むテストでは、単体hitの状態保持・連鎖・一投単位の採点・次投準備まで確認しています。確率0%・100%で転倒結果を固定し、得点・履歴・終了の既存の期待値も検証しています。実行結果と変更理由は[DEVELOPMENT.md](DEVELOPMENT.md)を参照してください。

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

`services::field::create_field`が標準12本・共通フィールド・棒とGuiMolkkyを組み立てます。`controllers/app.rs`のControllerはそれらとチーム・MouseInputを所有し、Molkkyトレイトを通じて固定刻みの更新・停止確認・採点・交代を接続します。画面全体はapp controller、フィールド・棒・スキットルの表示制御はcontrollersの各ファイルが担当します。`app.rs`はDioxusの状態とDOM接続の寿命を管理します。全チームで同じフィールドと棒を共有し、現在位置はGuiMolkkyの物理ワールドに保持します。操作用の初期投擲距離は1mです。高さ方向の転倒は物理サービスの力積による近似です。

表示範囲はx=-0.3〜0.3m、y=-0.15〜1.35mで固定し、物体の移動による拡大縮小は行いません。`GuiField::BOUNDS`と`WALL_THICKNESS`を描画と物理で共用し、外周の厚さ2cmの壁で棒・スキットルが反射します。反発係数は0.5で、既存の減衰と合わせて停止します。壁と反射はこのGUIの取り決めです。

### GUIの検証

```sh
cd rust_molkky
cargo test --locked --features gui_2d
cargo test --locked --no-default-features --features gui_2d
cargo clippy --locked --all-targets --features gui_2d -- -D warnings
# Controllerの入力・進行を絞って確認する場合
cargo test --locked --features gui_2d --lib gui_2d::controllers
```

Webを起動した状態で、実DOMの入力確認も実行できます。次はLinux用の例です。Playwrightは検証時だけ使用し、ゲームの実行依存には含めません。

```sh
cd rust_molkky
npm install --prefix /tmp/molkky-browser-tools --no-save playwright
/tmp/molkky-browser-tools/node_modules/.bin/playwright install chromium
NODE_PATH=/tmp/molkky-browser-tools/node_modules node tests/gui_2d/app.js
```

既存Chromiumを使う場合はCHROMIUM_PATH、別の起動URLを使う場合はGUI_URLを指定します。Chromiumでcapture・取消・リサイズ・領域外解放・採点・失格・勝利・終了後の入力無効化を確認しました。Linuxデスクトップは仮想画面で起動・投擲・交代を確認済みです。Windows/macOSの実機確認は行っていません。

### GUI 2Dの描画部品

`src/gui_2d/view/`にDioxusのElementを返す関数を実装しています。`gui_2d` featureを有効にすると、`gui_2d::view`から`field`、`skittle`、`stick`、`speedometer`、`scores`、`result`、`aim`を利用できます。

フィールドはworld座標の範囲と子要素を受け取り、SVGで上から見た表示を作ります。棒・スキットルの位置や寸法はappが渡します。得点表は既存Teamを読み取り、結果表示は判定済みの勝者を受け取ります。aimは照準の線と目標を表示します。

```sh
cd rust_molkky
cargo test --locked --features gui_2d --test gui_2d
cargo build --locked --features gui_2d
```

描画はdioxus-ssrによるHTML/SVG出力と、Chromium・LinuxのWebKitデスクトップで確認しています。既存CUIの起動方法は変わりません。

### GUI 2Dの投擲サービス

`gui_2d::services::GuiMolkky`はRapier2Dの状態を保持し、当たり判定・物理更新は`impl Molkky for GuiMolkky`に実装します。固有の`impl GuiMolkky`、physics.rs、エンジン専用traitはありません。services::field::create_worldにGuiStick、投擲位置、`(Id, [x, y])`の配置一覧を渡して生成します。座標・寸法はm、速度はm/s、角度はラジアンです。services::field::create_fieldが標準の個体と配置を生成して渡します。

| API | 動作 |
| --- | --- |
| aim | 投擲方向上の候補を返す。候補外も物理ワールドには残す |
| fly | 更新後のGuiStickを受けて1/60秒だけ飛行を進め、各個体の最大接触力積を記録する。棒と連鎖するピンが動いている間true、完了時はfalseを返す |
| hit | Stickと1本のSkittleを受け、更新後の個体だけを返す。物理時間は進めない |
| chain_hit | 転倒元と対象を受け、Rapierが蓄積した対象の転倒状態を返す。CUIの確率は使わない |
| stand_up | 対象個体の位置・Id・番号を保ち、角度・速度・角速度・今回の接触記録とStandを復元する。棒や他の個体は変更しない |

flyとstand_upはcoreのMolkkyトレイトの抽象です。flyの戻り値で継続/完了を伝えるため、停止確認専用のメソッドは不要です。stand_upは具象実装が必要な位置・角度の復元方法を決めます。今回の2Dでは倒れた場所で立て直します。coreに座標・Rapier型は追加しません。描画用の位置と回転はcontrollers::poseがIdから読み取ります。

GuiStick::throw(speed)は速度を更新したGuiStickを返します。物理ボディへの反映はMolkky::flyの最初の更新で行います。flyが棒を投擲位置へ戻し、回転・角速度・初速を反映します。飛行中は初速を上書きしません。完了後の次のflyは次投を始めるので、呼び出し側はfalseを受け取ったら次の解放まで更新を止めます。飛行開始前の棒の表示はcontrollerが投擲位置へ合わせます。

app controllerが固定時間ごとにflyを呼び、物理更新後に各Skittleへhitの返却値を反映します。接触力積はflyが蓄積し、Fallに変わった個体はその投擲中ずっとFallを保ちます。描画はモデルの状態とposeを読むだけで、hitを呼びません。flyがfalseを返した最後の更新も反映してから、今回のFall一覧を一度だけcalc_scoreへ渡し、coreの得点表・勝者・次手番の結果を反映します。固定刻み未満のフレームでは採点しません。継続する場合は各個体をstand_upで立て直し、移動後の位置とIdを保って次の照準を開始できます。

転倒は高さ方向の横倒しを接触力積で近似します。半径・質量はGuiSkittle、棒の半径・長さ・質量はGuiStickの関連定数に置き、配置・物理生成・描画で共有します。時間刻み・共通減衰・接触による転倒閾値はGUIサービスのシミュレーション定数です。時間刻みはservices::TIME_STEPから参照します。hitとchain_hitは個体間の相互作用なのでサービスへ置きます。物理テストでは高速衝突、弱い衝突、連鎖衝突、停止・再投擲を確認しています。

### GUI 2Dの入力と進行

`src/gui_2d/controllers/mouse.rs`に、`screen_to_world`と`MouseInput`を実装しています。座標変換はSVGの中央寄せ・アスペクト比維持・Y上向きに合わせ、余白や領域外を拒否します。入力の矩形はborderを除いたSVG描画領域のCSSピクセル値です。

MouseInput::default()はマウスの入力状態だけを用意します。press・move_to・preview・release・cancelで、左ボタンの所有・照準・時間ベースの速度（0〜10m/s、往復2秒）・解放・取消を扱います。previewとreleaseは目標座標と速度を返し、coreの棒・チーム・採点・物理ワールドには依存しません。領域外では最後の有効な照準を保持します。

services::teams::from_namesが初期チームを生成します。controllers/app.rsのControllerはservices::field::create_fieldの生成物を受け取り、最初の手番を設定します。進行状態のPhaseはこのcontrollerに置き、Ready・Charging・Simulatingを表します。Finishedや勝者のフラグは保存せず、is_finished()はcoreのnext_teamの結果を参照します。勝者はjudge_gameで求め、最後の生存者、全失格、参加者なしも既存ルールに従います。

app.jsはDOMのpointer captureと時刻通知を行う接続コードです。appが受信イベントをControllerへ渡し、マウス処理はMouseInputが担当します。待機中・未終了の場合だけ押下を受け付け、解放時の目標を設定し、GuiStick::throw(speed)で速度を更新します。Frameイベントではその棒をflyへ渡して固定刻みの物理更新、1本ずつの状態反映、完了の確認を行い、今回の全転倒個体を一度だけ採点します。update_game_stateは初期化と投擲完了時に既存のjudge_game・next_teamの判定を反映します。取消は入力状態と表示用の照準・速度を戻し、別pointerの取消や投擲中のfocus喪失では進行を戻しません。Webとdesktopで同じdocument::evalの接続を使います。

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
