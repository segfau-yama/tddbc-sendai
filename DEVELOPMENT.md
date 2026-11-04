# 開発支援方針の記録

## GUIサービスの固有implを撤去し制御をcontrollerへ集約（2026-11-04）

- 対象：GuiMolkkyの固有impl、GUIのapp/controller、fieldサービスと呼び出し元。`impl Molkky for GuiMolkky`は残し、当たり判定と物理処理をここへ集約する。ワールド生成はfieldサービス、表示位置の取り出しとviewの合成・イベント制御はcontrollers/各ファイル.rsへ移す。appはDioxusの起動・寿命管理だけを担う。
- coreのMolkkyにはstep/is_settled/reset_throwの既存GUI動作を抽象として追加する。即時判定のCUIは更新不要・停止済みをデフォルトとする。Rapier型・座標・表示情報はcoreへ入れず、新たなPhysicsトレイトや開始関数は作らない。Stickのthrowと採点・失格・勝敗・手番の規則は変更しない。
- 既存AppStateをcontrollers/app.rsのControllerへ移し、入力状態だけを持つ既存Controllerはcontrollers/mouse.rsのMouseInputへ改名する。controllers/field.rsは描画位置の取得とフィールド内の合成、stick.rsは投擲中/待機中の位置の選択、skittle.rsは位置・照準候補・モデル状態の描画への受け渡しを担う。メーター・得点表・結果の制御はapp controllerに置き、単にviewへ転送するだけの専用ファイルは作らない。新たな状態の複製は作らず、テスト本体も責務と同じcontrollers/各ファイル.rs・services/field.rsへ移し、既存の期待値を保つ。
- [x] 変更前の既存テストを確認する（GUI featureあり197件成功）。
- [x] trait経由で物理更新・停止・リセットを呼ぶテストを先に変更し、契約未定義のRedを確認する。
- [x] 固有implを削除し、生成・物理・表示制御・appの責務を移行する。
- [x] 既存の入力・転倒・連鎖・採点・二重加点防止・位置保持・再投擲のテストを維持する。
- [x] fmt/Clippy・feature分離・Web/desktopコンパイル・Web実操作・差分を確認する。
- Red：Molkky::step/is_settled/reset_throwを呼ぶよう先に既存テストを変更し、coreの契約にメソッドが未定義でコンパイルできないことを確認した（/tmp/molkky-controller-red.log）。
- Green/Refactor：cargo test --features gui_2dは197件、cargo testは144件成功。既存の進行処理は命名・配置以外の差分がないことを比較した。元のテストはcontrollers/app.rs・mouse.rs、ワールド生成の拒否条件はservices/field.rsへ移し、期待値を維持した。各描画controllerはモデルと物理状態を参照し、個体を重複所有しない。位置の取得も物理更新・再判定は行わない。
- 物理用の定数はGUIサービスのモジュールへ移した。coreのMolkkyへRapier用の定数や描画用のposeを追加せず、object安全性も維持する。directionとresolve_hitはそれぞれ2箇所から使う非公開の共通計算として残した。ワールド生成はfield::create_worldに移し、標準配置の生成元・任意配置の利用元が同じ構築処理を使う。
- 検証：cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、coreのみ/GUIのみのcargo check、Web向けcargo check/build、desktop向けcargo check、git diff --checkが成功した。desktopは既存の/workspace/.cloud-tools/desktopをPKG_CONFIG_SYSROOT_DIR/PKG_CONFIG_PATHで参照した。Chromiumのtests/gui_2d/app.jsでDOM入力・capture・取消・領域外解放・採点・失格・結果・固定範囲・反射を確認した（/tmp/molkky-controllers-browser.log）。デスクトップ実機の操作は今回は未確認。

## Stick側のThrowトレイトを実装（2026-11-04）

- Throw<Input>をcore/models/stick.rsに定義し、確率f64を受けるCuiStickと速度f32を受けるGuiStickへ実装する。throwはIdと他の属性を保って新しいSelfを返す。更新値は呼び出し側が指定し、確率や速度をランダムに生成する処理は追加しない。
- Molkkyのthrow/ThrowOutputとCUIの確率getterとしてのthrowを撤去する。Stick側のthrowはワールドを所有・更新せず、appは入力で確定した速度を反映した棒を既存のGuiMolkky::stepへ渡す。最初のstepだけ初速を設定し、停止後はreset_throwまで再投擲しない。ユーザーの追加指定に従いstart_throwなどの開始用関数は追加しない。
- [x] 変更前の既存テストを確認する（GUI featureあり195件成功）。
- [x] CUI/GUIの更新値・元の値の不変性・Idと照準の保持をテストしRedを確認する。
- [x] 共通トレイトと具象実装、core契約・app・既存テスト・READMEを移行する。
- [x] 物理更新の最初だけ初速を反映し、途中の入力で方向・速度を上書きしない。停止後は再開せず、立て直して次投へ進める。
- [x] 固定刻み未満のFrameでは採点せず、停止後の採点は一度だけ行う。
- [x] 全テスト・fmt/Clippy・feature分離・Webコンパイル・実操作・差分を確認する。
- Red：先に追加したモデルのテストは、Throwの未定義・throwメソッド未実装によるコンパイル失敗を確認した（/tmp/molkky-stick-throw-red.log）。既存stepへStickを渡す契約への移行も、先にテストを変更して引数・戻り値の不一致を確認した（/tmp/molkky-stick-step-red.log）。
- Green/Refactor：cargo test --features gui_2dは197件、cargo testは144件成功。CUIサービスの確率getterのテストはモデルの不変更新のテストへ移し、更新後の確率がhit・連鎖へ渡る検証も維持した。GUIは既存のactiveフラグを初速の未反映を示すreadyへ置き換え、停止状態はRapierのsleepから求める。appは実際に物理更新した後だけ停止・採点を確認する。開始用ヘルパーやモデルの投擲状態enumは追加していない。
- 検証：cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、coreのみ/GUIのみのcargo check、Web向けcargo check/build、git diff --checkが成功した。Chromiumのtests/gui_2d/app.jsでDOM入力・capture・取消・領域外解放・採点・失格・結果・固定範囲・反射を確認した（/tmp/molkky-stick-throw-browser.log）。検証用8081番のサーバーは終了した。デスクトップ実機の操作は今回は未確認。
- デスクトップ向けcargo checkも成功した。初回のGLib未検出は既存のローカル依存を参照する環境設定で解消した：PKG_CONFIG_SYSROOT_DIR=/workspace/.cloud-tools/desktop、PKG_CONFIG_PATH=/workspace/.cloud-tools/desktop/usr/lib/x86_64-linux-gnu/pkgconfig:/workspace/.cloud-tools/desktop/usr/share/pkgconfigを指定し、cargo check --no-default-features --features gui_2d_desktopを実行した。

## ThrowOutputとStick側のThrowトレイトを評価（2026-11-04）

- 対象：82eedf9のcore::Molkky::throw、core/GUI/CUIのStick、GuiMolkky::throw/reset_throwとappの解放処理。設計相談のみで、ソースやAPIは変更しない。
- 更新対象がStick自身の投擲状態であれば、ThrowOutputをなくしてSelfの更新値を返すThrowトレイトをStick側へ実装する案は妥当。例は `trait Throw: AsRef<Stick> + Sized { fn throw(&self) -> Self; }`。これは状態遷移の契約であり、物理ワールドの処理までモデルへ移す指定ではない。
- 現状のcore::StickはIdだけ、GuiStickは目標と速度、CuiStickは命中確率を保持する。目標・速度は投擲の解放までに既に設定されており、現在のthrowはStick自体のフィールドを更新していない。何が変わったStickを返すのかを定めず、同じ値を返すためだけにtraitや新しい状態フラグを追加しない。
- GuiMolkky::throw:L219はselfが所有するワールドをリセットし、棒ボディの回転・速度と計算中の状態を更新する。引数にStickしかなくてもselfのワールドへ作用しているため、この物理反映はサービス側に置く。Stickへワールドの所有・参照を追加して処理全体を移さない。
- 責務案：Stick側のThrowは必要になった個体自身の更新を返し、GuiMolkkyはそのモデルを物理ワールドへ反映して投擲を開始する。appがこの順序を接続する。無効な方向/速度を拒否する現在のboolの役割も、受付条件または状態の契約として維持する必要がある。
- CUIのthrowは確率を返すだけでrunからは未使用。GUIは物理開始なので、ThrowOutputは型の違いを扱えても動作の意味を揃えていない。再設計時はこの意味の不一致を解消し、CUIのgetterを共通の投擲動作として残すためだけの抽象を避ける。
- [x] 実装と実行経路を静的に確認し、個体更新と物理反映の境界を記録した。
- [x] 文書差分をgit diff --checkで確認した。実装・動作テストは行っていない。

## GUI個体の定数を所有モデルへ移動（2026-11-04）

- GuiMolkkyへまとめていた個体固有の定数を移す。半径と質量はGuiSkittle::RADIUS/MASS、棒の半径・長さ・質量はGuiStick::RADIUS/LENGTH/MASSとする。値と単位（m/kg）は変更しない。物理生成・配置・描画・テストは同じモデル定数を参照する。
- 初期実装ではRapier初期化の近くへ定数を集めたが、利用場所と所有すべき責務を混同していた。個体の属性はモデル、ワールドの時間刻み/共通減衰と個体間の転倒閾値はGuiMolkkyへ配置する。
- hitはStickとSkittle、chain_hitは2本のSkittleの相互作用なのでサービスへ保持する。throwはStickのみを引数とし、モデルへhitを移したりthrowへSkittleを追加したりしない。coreへGUIの寸法/質量は持ち込まない。
- [x] 前段の195件のテスト成功を変更前の基準として確認した。
- [x] 定数定義とすべての参照元・READMEを移行する。旧SKITTLE_*/STICK_*定義と参照はsrc/testsから撤去した。
- [x] 既存テスト・fmt/Clippy・feature分離・Webコンパイル・差分を確認する。値の移動のみのため新規テストやRedは設けない。
- 検証：cargo test --features gui_2dは195件成功。cargo clippy --all-targets --features gui_2d -- -D warnings、cargo fmt --check、cargo check --no-default-features、cargo check --no-default-features --features gui_2d_web --target wasm32-unknown-unknown、git diff --checkが成功した。質量・寸法の数値、衝突計算、hit/chain_hit/throwの署名・配置は変更していない。

## hitのbool撤去・chain_hit・stand_upを実装（2026-11-04）

- hitのboolを削除し、状態を含む個体を返す。CuiSkittleが連鎖確率を保持できるよう、aim/hit/chain_hit/stand_upの戻り値を関連型Self::Skittleへ揃える。coreの値を復元するstand_upのデフォルト実装にFrom<core::Skittle>を使う。
- CuiSkittleはcore::Skittleと連鎖用確率を合成する。ユーザー回答に従い、初回転倒時は投げたStickの命中確率を保持する。chain_hitは転倒元の確率から0.2を引いて0で止め、成功した連鎖先へその減衰後の確率を引き継ぐ。隣接探索は追加せず、呼び出し側が連鎖対象を指定する。
- 立て直しはcoreの共通メソッドでFall→Standへ戻す。CUIのFromで確率も初期値へ戻す。GUIの物理リセットはreset_throwへ改名して役割と名前を分け、物理の連鎖はRapierの接触結果に従う。採点規則は変更しない。
- [x] 既存テストを確認する（187件成功）。
- [x] 共通stand_upの契約テストを先に追加しRedを確認する（メソッド未実装によるコンパイル失敗）。hit/aimの戻り値を関連型へ移行し、確率を保持するCuiSkittleへ統合した。
- [x] core/CUI/GUIと呼び出し側を移行し、状態・Id・採点の既存検証を維持する。
- [x] 連鎖成功/失敗/0%/既転倒/確率減衰と立て直しを決定的な乱数で確認する。
- [x] 全テスト・fmt/Clippy・feature分離・Web・差分を確認しcommit/pushする。
- Green：cargo test --features gui_2dは195件成功。hitは個体のみを返し、共通stand_upはId/番号を保ってStandへ戻す。CUIは連鎖確率も0へ戻り、元の個体は変更しない。GUIはreset_throwで物理状態を準備した後、共通stand_upで個体状態を戻す。
- CUIの連鎖テストでは固定の乱数源を使い、Stickの80%を初回転倒へ保持、次は60%・40%と減衰、失敗時の状態保持、0%で停止、既転倒の再判定防止、同番号の別Idとまとめて採点する契約を確認した。CuiMolkkyの乱数源は非公開の関数ポインターで、公開のStrategyやテスト専用setterは設けていない。決定的なテストのためCUIサービスのテスト登録をソースのcfg(test)へ移し、テスト本体は同じtests/cui/services/molkky.rsに維持した。
- 追加のRed→Green：0.8から0.2を繰り返し引く実装は、最後に5.55e-17が残って固定乱数0で余分な個体を倒すことをテストで確認した。パーセント単位へ変換して20ポイントを引く計算へ変更し、80→60→40→20→0で停止することを確認した。CHAIN_DECAY_PERCENTはimpl内の定数とした。
- GUIのchain_hitは確率を持ち込まず、転倒元がFallの場合にワールドが蓄積した対象の転倒状態を読む。hitと共通の判定部分を複数箇所から使い、stepは呼ばない。stand_upでモデルだけを立てても物理位置が変わらないことも確認した。連鎖対象の自動探索・CUI入力ループへの自動連鎖は今回の範囲外であり、APIの引数で指定する。
- 検証：cargo test、cargo test --features gui_2d、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、cargo check --no-default-features、cargo check --no-default-features --features gui_2d、Web向けbuild/checkとJS構文確認、git diff --checkが成功。途中のClippyのテスト名警告は日本語の命名を修正し解消した。
- Web：検証用8081番で起動し、Chromiumのtests/gui_2d/app.jsが成功（capture・取消・領域外解放・採点・失格・結果・固定範囲・反射）。ログは/tmp/molkky-chain-browser.log。検証サーバーは終了した。デスクトップ実機は今回は未確認。READMEのAPI例・確率の意味・stand_upとreset_throwの責務を更新した。

## 単体hitとGUIの個体状態反映を実装（2026-11-04）

- 前回の機能表を実装する。hitはStickと1本のSkittleから更新後のSkittleとStand→Fallの新規転倒boolを返す。coreはイミュータブルなモデルと採点・勝敗・次手番の既存ルールを維持する。
- GUIのワールド・aim/throw/hit/step/停止/位置はGuiMolkkyに置く。appが各フレームの物理更新後に個体状態を反映し、描画はその状態を読む。停止時は1投のFall一覧を一度だけ採点する。継続時は位置・Idを維持して物理と個体の状態を次の照準までに立て直す。
- CUIも新しいcore契約へ移行し、狙った各個体を一度ずつ判定して今回の転倒一覧をまとめて採点する。新しいStrategy/Physics trait、physics.rs、値オブジェクト用GUIモデルは作らない。
- [x] 変更前の既存テストを確認する（GUI featureあり181件成功）。
- [x] 単体の不転倒・転倒・再問い合わせの契約テストを追加してRedを確認する（旧APIのスライス引数とVec返却が新しい単体/タプルの期待と合わずコンパイル失敗）。
- [x] core/CUI/GUIと呼び出し側・既存テストを移行してGreenを確認する。
- [x] 描画中の再判定を撤去し、早期のFall保持・連鎖・複数本採点・二重加点防止・次投準備を確認する。
- [x] fmt/Clippy・CUI/GUIテスト・feature分離・Webコンパイル・差分を確認しcommit/pushする。
- Green：cargo test --features gui_2dは187件成功。既存の一覧単位の期待値はテスト側で単体hitを集約して維持し、単体契約そのものも直接検証した。GUIのhitが物理時間を進めないこと、既にFallなら新規転倒falseとなること、無関係のStickなら状態を変えないことを確認した。
- appは固定刻みのstep後に全個体へhitの状態を反映し、早期のFallを保持する。停止時には新規転倒boolだけでなく今回の全Fall一覧を採点する。5番と8番の連鎖を2点・1投として記録し、次の手番へ進め、次の照準ではStandに戻る実エンジンの進行テストが成功した。表示はモデルを読み、hitや状態の再構成を行わない。
- GuiMolkky::stand_upはId/個体位置を保ち速度・回転・接触記録を消し、棒を投擲位置へ戻す。appは継続時にこれを呼び、core個体もStandへ戻す。単独のGuiMolkky::throwも従来の次投契約を保つためstand_upを使い、開始側の重複した位置・角速度初期化は除いた。関連27件の物理/描画テストで整理後も成功を確認した。
- coreのcalc_score/judge_game/next_teamとScoreBoardのルールは変更せず、GUIのwinnerによるnext判定の重複分岐だけを撤去した。Controllerとservicesの初期化責務は前回の分離を維持した。READMEの例とhitの新しい契約を更新した。
- 検証：cargo test、cargo test --features gui_2d、cargo test --features gui_2d --test gui_2d、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、cargo check --no-default-features、cargo check --no-default-features --features gui_2d、Webのbuild/check、JS構文確認、git diff --checkが成功。
- Web実操作：一時ディレクトリ/tmp/molkky-browser-toolsへPlaywrightを用意し、既存の/usr/bin/chromiumでtests/gui_2d/app.jsが成功。capture・領域外解放・取消・採点・失格・終了表示・固定範囲・反射を確認した。リポジトリの依存関係は増やしていない。検証用Webサーバーは終了済み。ログは/tmp/molkky-single-hit-browser.log。デスクトップ実機は今回は未確認。

## GUI機能の責務分割と単体hitの設計レビュー（2026-11-04）

- 対象：d0b82b8のcore/services/molkky.rs、core/models/score.rs・team.rs、GUIのapp/controller/services/models/viewと既存テスト。今回は設計レビューのみで、Rust/JSやトレイトの契約は変更しない。
- 最新の指示を採用する。physics.rsへの分離を必須とせず、GuiMolkkyがRapierのワールドを所有し、coreのaim/throw/hitの具象実装で物理処理を担う。coreの採点・勝者・次手番のデフォルト実装を再実装しない。前回の「採点・手番をappへ移した」という説明は不正確で、現状もcoreを呼んで結果を反映している。

| 機能 | 担当・API案 | 境界・結果 |
| --- | --- | --- |
| Web/desktop起動 | bin/gui_2d.rs、app::App | レンダラー起動とDioxusの寿命管理。 |
| チーム生成 | gui_2d/services/teams.rs | 名前からcore::Teamを生成。入力順・個体Idを保つ。 |
| 個体・共通Field・Stickの準備 | gui_2d/services/field.rs | 初期個体と標準配置を決め、GuiMolkkyへ渡して組み立てる。 |
| 物理ワールド初期化 | GuiMolkky::new | ボディ・壁・形状・質量・減衰・CCD、Id対応を生成。採点しない。 |
| マウス押下・移動・解放・取消 | controller::Controller | pointer所有と入力の解釈だけ。ゲームの継続可否は決めない。 |
| 画面座標からフィールド座標への変換 | controller::screen_to_world | SVGの拡縮・余白・Y方向を扱う。物理位置を変更しない。 |
| 蓄力と解放時の速度 | Controller::preview/release | 経過時間から目標と速度を返す。棒のモデルへ反映するのはapp。 |
| DOM capture・実寸・時刻通知 | app.js/アプリ接続部 | ブラウザーAPIとの橋渡しのみ。coreのルールを持たせない。 |
| 入力の受付とGUI操作段階 | app | coreの終了判定とReady/Charging/Simulatingから受付を制御。 |
| 照準候補 | GuiMolkky::aim | 物理位置から候補を返す。候補外の個体もワールドに残し連鎖衝突を可能にする。 |
| 投擲開始 | GuiMolkky::throw | 棒の方向・初速を設定し開始の成否を返す。全シミュレーションを同期的に回さない。 |
| 物理更新を呼ぶ時刻 | app | 経過時間を蓄積し固定刻みのstepを必要回数呼ぶ。 |
| ワールドを1刻み更新 | GuiMolkky::step（具象API） | 全ボディを一緒に進めて衝突を解く。個体ごとのhitの中でstepしない。 |
| 衝突・反射・減速・転倒材料の蓄積 | GuiMolkky/ワールド | 接触力積、連鎖衝突、今回の個体別最大値を保持する。単位・転倒閾値はここへ閉じ込める。 |
| 1本の状態と転倒結果の取得 | GuiMolkky::hit（変更案） | Stickと1本のSkittleを受け、更新後のcore::Skittleと必要なら新規転倒boolを返す。 |
| 個体一覧への状態反映 | app | 各hitの返却値をモデルへ反映し、描画に転倒判定を再実行させない。 |
| 停止の判定 | GuiMolkky::is_settled（具象API） | ワールド全体のsleep等を見る。採点や手番変更はしない。 |
| 1投分の得点計算 | core::Molkky::calc_score | 今回倒れた個体をまとめて渡す。1本なら番号、複数なら本数。 |
| 履歴・合計・超過補正・失格・50点到達 | core::ScoreBoard::add/new/state | coreモデルの規則を使う。GUIでルールを再計算しない。 |
| 勝者の決定 | core::Molkky::judge_game | Wonと最後の生存者の既存ルールを使う。 |
| 次の手番の決定 | core::Molkky::next_team | coreが次のTeamまたはNoneを返す。 |
| 得点・勝者・手番の更新結果の保持 | app + coreのwith_score_board/with_turn/with_win | coreが返したPoint・Teamの結果を反映する。決定規則を複製しない。 |
| 次投の準備・立て直し | GuiMolkkyとappの接続 | 個体Idと移動後の位置を保ち、速度・回転・今回の接触記録と個体のFallを戻す。次の照準開始前に整合させる。 |
| 位置・回転の読み取り | GuiMolkky::pose（具象API） | 位置の正本はRapierだけ。modelsへ同じ位置を別保存しない。 |
| Field・Skittle・Stick・照準の描画 | view/field.rs・skittle.rs・stick.rs | 渡されたモデル状態と位置を表示。物理更新・hitを呼ばない。 |
| 速度メーター・得点表・結果の描画 | view/speedometer.rs・score.rs・result.rs | 入力計算値・coreの履歴/状態・判定済み勝者を表示。 |
| 終了後の入力無効化・接続終了時の取消・後始末 | app | coreのnext_team/judge_gameの結果とDioxusの寿命へ従う。勝者フラグを別保存しない。 |

- 単体hitの契約案は `fn hit(&self, stick: &Self::Stick, skittle: &Self::Skittle) -> (Skittle, bool);`。coreモデルのイミュータブルな方針を保ち、新しいSkittleを返す。boolは「今回の更新でStandからFallへ変わったか」と定義すれば、既にFallの個体へ再度問い合わせたときfalseになり、現在の状態とは意味が異なる。単に「更新後にFallか」ならstateから取得できるためboolは不要。独自のHitResult型は先に作らない。
- StickとSkittleのcoreデータだけでは接触の判定材料が足りない。GuiMolkkyのselfにあるワールドと今回の接触記録を参照する。Stick引数は投擲の文脈を指定し、棒との直接衝突だけに限定しない。ピン同士の連鎖も今回の転倒に含める。
- 物理はワールド単位、状態反映は1本単位、採点は1投単位。stepは毎tick全体を進め、hitはその結果を読む。appは投擲中にFallを保持して停止後にその一覧を一度だけcalc_scoreへ渡す。5番と8番が倒れた例で、単体hitの結果を個別採点して13点にせず2点とする。新規転倒boolがその更新でfalseでも、既にFallの個体を今回の採点一覧から落とさない。
- step/is_settled/poseは現行coreトレイトにないGUI固有の具象APIとして残す。非同期に進む物理をaim/throw/hitの3メソッドだけへ押し込めず、1本ずつhitするたびに時間を進める設計にしない。必要性のないPhysics traitや各APIの単独転送ラッパーは追加しない。将来physics.rsへ実装を移す場合も内部の整理として扱い、今の提案の必須条件にはしない。
- 所有はappがTeam一覧・GuiField・GuiStick・GuiMolkkyを一つずつ持ち、GuiMolkkyがワールドを一つ所有する。GuiSkittle/GuiTeam/GuiScoreBoardはcoreへの委譲型であり、採点状態や物理ワールドの別の正本を持たせない。coreへDioxus/Rapier/座標を追加しない。
- 指摘（中）gui_2d/app.rs:L184: shrink: update_game_stateのwinnerによるnextの分岐はcore::next_teamの終了判定と重複する。next_teamを直接呼ぶ1行へ短縮可能（現行5行→1行）。coreはWonと最後の生存者の場合にNoneを返すため、同じ規則をGUIへ再記述しない。
- 指摘（中）gui_2d/app.rs:L222 / services/molkky.rs:L222: shrink: 新しいhit契約を採用する場合、描画中に再判定する処理を削除し、appで更新した個体状態を描画へ渡す。インターフェース変更を伴うため今回は適用しない。
- [x] 現行コードとcoreの契約・呼び出しを静的に照合した。
- [x] 最新の指示に合わせた機能表、単体hitの意味、既存APIとの変更点を記録した。
- [ ] 実装する場合：1本の不転倒/転倒/再問い合わせでId・番号と新規転倒boolが整合することを確認する。
- [ ] 実装する場合：連鎖・同じ番号の別Id・複数本の採点・二重加点防止・次投のStand復帰・物理の更新回数を確認する。
- 実装/動作テストは行っていない。文書だけをgit diff --checkで確認する。上記の4行短縮は静的な差分比較であり、物理API再構成全体の削減行数は未算定。

## GUIの入力・初期化・進行の責務を分離（2026-11-04）

- 指示：Controllerはマウス入力だけを担当し、チーム生成とField/Skittle/Stickを使う初期化はservicesへ移す。一度だけ使う取次ヘルパーは撤去する。レビュー指定に加えて削除・変更が明示されているため、今回は修正まで実施する。
- Controllerはpointer所有・座標変換・蓄力・解放・取消のみを扱い、Team/GuiMolkky/物理更新/採点/終了判定を持たない。appが入力可否を判断し、GUIの状態・時計とcore/物理への接続を所有する。
- services/teams.rsは名前からTeamを生成する。services/field.rsは標準12本・配置・棒・物理ワールドを構築する。GuiFieldモデルは渡されたcore値を保持し、標準配置を自動生成しない。
- 削除対象：Controllerの参照用getter、単独呼び出しのupdate/cancel_pointer、標準配置だけを転送するヘルパー。入力APIや描画・投擲完了など具体的な振る舞いの単位は、呼び出し数だけを増やすために分割しない。
- [x] 変更前の全Rustテストを確認する（GUI featureあり179件成功）。
- [x] チーム生成・フィールド構築のテストを先に用意しRedを確認する（services::field/teams未定義でコンパイル失敗）。
- [x] 初期化をservicesへ、入力以外の進行をappへ移す。既存テストの検証を維持して移動する。
- [x] 入力・取消・二重加点・勝敗・標準配置・CUI回帰、fmt/Clippy、feature分離/Webコンパイルと差分を確認する。
- Green：cargo test --features gui_2dは181件成功。座標/蓄力/取消はcontroller、初期化/進行/採点/描画はapp、標準個体と物理配置はservices/fieldへテストを移した。名前からの別Id生成とapp側の入力拒否を明示するテストを追加した。入力結果はGuiStickではなく目標座標と速度になるため、棒のId保持はapp側の既存の取消・実投擲テストで確認する。
- 移行中のテストのOptionと座標配列の型不一致を修正した。これを仕様のRedとは扱わない。標準配置は返却用配列の順序を検査する代わりに、元の個体Idに対応するRapier上の全12座標を同じ期待値で直接確認する。
- レビュー：Controllerからドメイン・物理・進行状態の所有を撤去した。teams/field/stick/phase/rulesの取次getter、update、cancel_pointer、GuiFieldのfield getter/initial_positions/標準生成Defaultを削除した。servicesの生成関数は初期化そのものを担当するAPIとして残し、GUIで現在使う固定の名前をTeamへ変換する。coreの規則とGuiMolkkyの物理計算は変更していない。
- 描画の組み立てと固定結果からの投擲完了はそれぞれ実際の処理とテスト境界を持つためapp側へ保持した。単独の呼び出し数を増やすための分割・新規のtrait/汎用基盤は追加していない。AppStateはapp内の進行コンテキストであり、新たなドメインモデルではない。
- DOMのpointer/時刻通知はappの接続責務なので、controller.jsをapp.jsへ改名し、対応するDOMテストもtests/gui_2d/app.jsへ移した。JSの処理内容は変えていない。
- 検証：cargo test、cargo test --features gui_2d、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、cargo check --no-default-features、cargo check --no-default-features --features gui_2d、cargo check --no-default-features --features gui_2d_web --target wasm32-unknown-unknown、JS本体/テストのnode構文確認、git diff --checkが成功。
- 未実施：ブラウザー/デスクトップ実操作。この時点では8080番にサーバーがなく、以前のPlaywright用パスにもモジュールがなかった。SSRの表示と実Rapierの停止・採点・入力拒否はRustテストで確認した。

## controller.jsを用意した理由とDioxusへの集約方針（2026-11-04）

- 対象：gui_2d/controller.js、app.rs、controller.rsと採用中のDioxus 0.7.10の公式クレートソース（dioxus-htmlのpointer/generated/mounted、dioxus-webのpointer、dioxus-desktopのelement）。質問への説明のみで、ソースは変更しない。
- 元の理由：Webとdesktop WebViewで同期的なpointer capture、SVGの実寸とborder計測、window blur/document visibilitychange、requestAnimationFrameの時刻通知と後始末を共通化するため、document::evalのDOMブリッジとして分離した。採点・勝敗・物理計算はJSに置いていない。
- Dioxus標準のonpointerdown/onpointermove/onpointerup/onpointercancel/onlostpointercaptureとPointerDataの座標・button・pointer_idで通常の入力は扱える。JSでaddEventListenerを全て登録したことは、Dioxusがイベントを処理できないためではない。
- 制約：0.7.10の共通MountedData/RenderedElementBackingにはsetPointerCapture/releasePointerCaptureの操作がなく、DOM寸法取得はget_client_rectとして提供される。Webならweb-sysからcapture APIをRustで呼べるが、同じ呼び出しをnative desktopへ流用できない。desktopの標準APIも内部ではJSを介しており、Rustで書けることとブラウザーAPIへの橋渡しが不要なことは区別する。領域外解放・取消・同期captureを維持して共通APIだけへ移せるとは未検証のまま断定しない。
- 整理案：通常のイベントと入力可否の判断はDioxus/Rustへ寄せ、DOM固有の操作だけを最小限の補助へ絞る。フレーム更新の状態・寿命管理はRust側へ寄せられるが、use_future自体はタイマー/画面同期APIではなく、経過時間の供給手段はWeb/desktop双方で確認する。window/document全体のイベントも要素のonblurと同一視しない。
- 優先度中：controller.js:L38のdata-phaseとownerによる入力可否判定はController側と重なる。Rustの進行状態を正本にして、DOMのcapture操作に必要な範囲へ縮小する案を優先する。JS全体を必須とする根拠はない一方、ファイルを削除しただけで既存の入力要件を維持できるとも扱わない。
- [x] 現行処理・開発記録・採用バージョンのAPIを静的に確認し、理由と縮小可能な範囲を記録した。
- [x] 文書だけの差分をgit diff --checkで確認した。動作テストと置き換えの試作は実行していない。

## GUIモデルをCUIと揃えGameの入れ物を撤去（2026-11-04）

- 範囲：GuiField/GuiStick/GuiSkittle/GuiTeam/GuiScoreBoardをcoreへの委譲型としてmodelsへ配置する。値オブジェクト・状態enumのGUI用ラッパーは作らない。GuiFieldへAsRef<Field>を追加し、新規の単純な型はFrom/AsRefだけを持つ。物理エンジンやcore APIは変更しない。
- game.rsはチーム・場・棒・GUI進行のデータを束ねるために作られ、勝利判定は元からcore::Molkkyが行っていた。現在Controllerだけがこの入れ物を所有するためGameを削除し、データはControllerへ直接置く。読み取り用の参照を公開し、モデルを重複所有しない。
- Controllerに残すInputPhaseはReady/Charging/Simulatingの操作段階だけ。Finishedは保存せず、is_finishedがcoreのnext_team結果から判定する（ScoreBoardのWon、失格、最後の生存者、参加者なしを既存ルールへ委譲）。採点ルールや勝者の独自フラグを新設しない。
- [x] 変更前のGUI/CUIテストを確認する（179件成功）。
- [x] GuiFieldのAsRef契約を既存の同一性テストで確認しRedを記録する（AsRef未実装のためコンパイル失敗）。
- [x] 委譲型追加、Game削除、Controller/app/既存テストの移行を行う。Game初期化のテストはControllerへ移し、検証内容を保つ。
- [x] 採点の一回性、終了後の入力拒否、空参加者、結果描画、CUI回帰、fmt/Clippy/feature分離/Webコンパイルを確認する。
- Green/Refactor：cargo test --features gui_2dは移行後も179件成功。Gameのテスト1件をcontrollerへ移したため内訳は変更したが、確認内容は削除していない。終了後にpointerを取得しないことも既存の勝利後入力テストへ明示した。
- 検証：cargo test、cargo test --features gui_2d --lib scoring、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、cargo check --no-default-features、cargo check --no-default-features --features gui_2d、cargo check --no-default-features --features gui_2d_web --target wasm32-unknown-unknown、git diff --checkが成功。
- レビュー：Game/GamePhase/保存するFinishedを撤去し、入力の3段階だけをcontrollerに置いた。得点・失格・最後の生存者の規則はcoreへ委譲し続ける。物理ワールドの所有や転倒近似、CUI/coreのソース、依存関係は変更していない。GUIモデルの値をcoreへ渡すためだけの複製処理は加えていない。
- 未実施：ブラウザー/デスクトップの実操作。デスクトップのネイティブビルドに必要なgtk+-3.0/webkit2gtk-4.1がこの環境ではpkg-configで揃わないため、今回はデスクトップ向けビルドは実行していない。

## CUIの値オブジェクト用ラッパーを削除（2026-11-04）

- 前回の「全公開モデルを委譲型にする」方針を修正する。Id/Name/Pointと状態enumはcoreの型を直接利用し、CUI側へ重複した型を設けない。
- CuiId/CuiName/CuiPoint/CuiScoreState/CuiSkittleStateと公開宣言を削除する。CuiField/CuiStick/CuiSkittle/CuiTeam/CuiScoreBoardは維持し、coreや入力・採点・描画処理は変更しない。
- [x] 参照先と変更前の既存テストを確認する。対象ラッパーは実行処理・テストで未使用だった。
- [x] 不要な型・ファイル・公開宣言を削除しREADMEを更新する。
- [x] 変更後の既存テスト、fmt、Clippy、差分を確認する。振る舞いを追加しないため新規テストやRedは設けない。
- 検証：変更前後のcargo test --features gui_2d（179件）、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、git diff --checkが成功。rgで削除した型への参照がsrc/testsに残っていないことを確認した。

## CUIモデルをcoreへの委譲型へ統一（2026-11-04）

- 範囲：独自のTeamsモデルを削除し、名前の検証・チーム一覧生成をservices/teams.rsのfrom_names/add_nameへ移す。入出力とrunはVec<Team>を受け渡し、一覧を包む別モデルは設けない。
- core/modelsの公開型すべてに対応するCuiXxxxをcui/modelsへ用意する。単純な委譲型はFrom<Core>とAsRef<Core>だけを実装し、coreのルールやgetterを複製しない。状態enumも再定義せずcoreの値を内包する。
- 既存Fieldはcore::Fieldを内包するCuiFieldへ改名する。標準12本の保持と確率付きCuiStickの動作を維持する。委譲型をcoreサービスへ渡すためだけの変換列や関連型追加は行わず、core APIとGUIは変更しない。
- [x] 変更前のGUI/CUIテストを確認する（178件成功）。
- [x] チーム検証テストをservicesへ移し、CuiFieldが元のFieldと個体を保持するテストを追加してRedを確認する（CuiFieldとservices::teamsが未実装のためコンパイル失敗）。
- [x] Teams削除、services移動、全委譲型、呼び出し側とREADMEの移行を実装しGreenを確認する。
- [x] GUI/CUIテスト、feature分離、fmt、Clippy、差分を確認する。
- 委譲型：CuiField/CuiId/CuiName/CuiPoint/CuiScoreBoard/CuiScoreState/CuiSkittle/CuiSkittleState/CuiStick/CuiTeam。新規の単純なFrom/AsRefにはcoreの処理を再実装するメソッドを加えていない。チーム集合とMolkkyへの受け渡しは既存のVec<Team>/Skittleを維持し、一覧の複製やcore側のAPI変更を避けた。
- 検証：cargo test --features gui_2d（179件）、cargo test、cargo check --no-default-features、cargo check --no-default-features --features gui_2d、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warnings、git diff --checkが成功。チーム名の1〜30文字・制御文字不可・trim・同名の別Id生成、標準12本、既存のゲーム進行テストは維持した。
- レビュー：modelsは全て対応するcore型を内包し、Teamsモデルと旧Field公開名は撤去した。core/GUI/依存関係は変更していない。ブラウザー・デスクトップ実機での操作確認は未実施。

## MolkkyのAsRef制約を実装（2026-11-04）

- 範囲：Stick/Skittleの関連型へAsRef制約を追加し、CUI/GUIの既存型で実装する。GuiSkittleの新設やField・物理処理の再構成は別段階とする。
- 契約：aim/hitは関連型のスキットル一覧を受け取り、coreのSkittle一覧を返す。採点・勝敗・手番と物理挙動は変更しない。AsMut/Derefや新しいtraitは追加しない。
- [x] 変更前の既存テストを確認する（cargo test --features gui_2d成功）。
- [x] 関連型経由で内側のモデルを参照するテストを追加し、Redを確認する（Skittle関連型未定義とStickのAsRef制約不足によるコンパイル失敗）。
- [x] 制約とAsRef実装を追加し、Greenを確認する。GUI/CUIとも現段階の関連型Skittleはcore::Skittleを指定し、core自身のAsRefも明示実装した。
- [x] GUI/CUIの回帰、feature分離、fmt、Clippy、差分を確認する。
- 結果：cargo test --features gui_2dの178件成功。cargo check --no-default-featuresと同--features gui_2d、cargo fmt --check、cargo clippy --all-targets --features gui_2d -- -D warningsが成功。dyn Molkkyを使う既存テストは新しい関連型指定へ更新した。
- 実装中にテスト側のSkittle import不足を修正した。このコンパイル失敗は仕様のRedとは区別する。デスクトップ実機・ブラウザーの操作テストは未実施（入力・描画処理は変更していない）。
- Web：cargo check --no-default-features --features gui_2d_web --target wasm32-unknown-unknown成功。

## Molkkyの関連型をcoreモデルへ委譲できる型に制限する設計（2026-11-04）

- 対象：core/services/molkky.rsとGUIモデル。設計相談のみで、ソースは変更しない。Ponytailの方針に従い、新しい独自traitより標準のAsRefを優先する。
- 推奨API案：`type Stick: AsRef<Stick>; type Skittle: AsRef<Skittle>;`とし、aim/hitの引数を`&[Self::Skittle]`へ変更する。GuiMolkkyではGuiStick/GuiSkittle、CUIでは既存の具象型を指定する。具象実装ではGUIのbodyハンドル等もそのまま読める。
- AsRefはcoreモデルへの共有参照を提供する契約であり、フィールドの内包や全メソッドの自動委譲をコンパイラが検査するものではない。Rustには任意のフィールドを持つ型を構造的に指定する制約はない。core自身を関連型に使う場合もAsRef<Self>の明示実装が必要。
- hitの戻り値は当面`Vec<Skittle>`を維持できる。転倒した個体のcore情報を返すことで、GuiSkittleやボディハンドルを採点用に複製する必要がない。具象モデルへFallを反映する処理は別途必要であり、AsRefだけで状態更新まで解決するわけではない。
- 内側を置き換える契約が実際に必要ならAsMutを追加する。AsRefは既存のwith_stateによる更新やGUI状態の同期を代行しない。Derefによる暗黙のメソッド解決は必要なく、委譲用macroや新しいStrategy traitも追加しない。
- 既存の採点・勝敗・次手番のデフォルト実装は維持可能。この案なら前回検討した物理メソッドのcore外への移動は必須ではない。ただし関連型追加と全実装の署名変更は必要。
- [x] 現行トレイトの関連型とGuiStickの合成を静的に確認した。
- [x] 文書差分をgit diff --checkで確認した。ソース変更・動作テストは行っていない。

## GuiSkittle追加と空のGuiFieldを前提にした設計レビュー（2026-11-04）

- 対象：126d12f時点のgui_2d/models、services/molkky.rs、app.rs、controller.rs、view。今回は設計案だけを整理し、ソースや既存APIを変更しない。レビュー記録はAGENTS.mdに従い保存する。
- 現在の複雑さはGuiSkittleという型の欠落そのものより、GuiFieldのcore::Skittle一覧、GuiMolkkyのcollider→Id表・力積表、Rapierの位置、描画で作る一時的なFall状態を毎回結合する構造にある。GuiSkittleはこの個体単位の対応と状態をまとめる目的で追加する。

### 推奨する所有とモデル

- ControllerはGameとGuiMolkkyを一つずつ所有する。Gameはteams/field/stick/skittles/phaseを持ち、Vec<GuiSkittle>の唯一の所有元とする。GuiFieldからskittlesを外し、フィールド境界・壁・投擲位置等の場の情報を扱う。空のフィールドを生成でき、全チームが同じ場と個体集合を共有する。
- GuiSkittleはcore::SkittleとRigidBodyHandleを合成する。Id・番号・Stand/Fallの正本は内側のcore::Skittle、位置・回転・速度・衝突形状の正本はRapierのWorld。GuiSkittleへ位置・角度の永続コピーや重複したis_fallenを追加しない。GuiStickもcore::Stick、対応ボディのハンドル、照準・初速を持つ形へ揃える。
- GuiMolkkyはWorldと投擲位置等の物理実行コンテキストを所有し、GameのGuiSkittle/GuiStickを借用して扱う。core::Skittleを別の一覧にも保存しない。coreモデルへRapierの型を入れない。
- ハンドルは作成元Worldに結び付く。Controllerの初期化でWorld・ボディ・GUIモデルを一緒に準備し、別Worldへのハンドル使い回しや、Game/GuiSkittleを独立にCloneして状態管理する形は避ける。未登録ハンドルのOption状態は先に設けない。

### 配置・描画・投擲の責務

| 対象 | 役割 |
| --- | --- |
| GuiField | スキットルを含まない場。境界、壁、投擲位置を定義する。 |
| 初期配置関数 | GuiFieldと個体集合を受け取り配置を計算する。標準配置をFieldの所有メソッドから切り出し、物理ボディ生成と混ぜない。変則配置は別の関数または配置データで選ぶ。 |
| GuiSkittle | coreの個体と物理ボディの対応、実際のStand/Fallを保持する。衝突の閾値や採点規則は持たない。 |
| GuiMolkky::aim | GuiStickとGuiSkittle一覧を読み取り、物理位置から候補のIdを返す。候補は描画用の一時結果とする。 |
| GuiMolkky::throw | 棒のボディへ方向・初速を設定して投擲を開始する。点数や手番は変更しない。 |
| GuiMolkky::step/hit | stepがWorldを1tick進め、必ずhitを呼んでそのtickの衝突を各GuiSkittleへ反映する。hitは閾値以上なら内側のSkittleをwith_state(Fall)で置き換える。Fallはその投擲中維持する。 |
| GuiMolkky::pose | ボディハンドルから直接位置・回転を読む。Idからcolliderを走査する結合処理をなくす。viewには座標など描画に必要な値を渡す。 |
| view::skittle | Skittleの番号・状態と渡された位置・寸法を描画する。hitを呼んだり、転倒一覧から表示用の状態を再構築したりしない。 |
| Controller | 停止後にFallのcore::Skittleだけを取り出してcoreへ採点を依頼し、ScoreBoardと手番・勝敗を更新する。継続時には立て直しを依頼してReadyへ進める。 |

- 転倒をGuiSkittleのcore状態へ各tickで反映するため、peak_impulsesの別表を削除できる。contactは停止前に消えることがあるのでhitを停止時だけ呼ぶ設計にはしない。collider→Idの別表も、各GuiSkittleのbodyに付いたcolliderから接触を調べる形へ置き換える。通常12本のため、必要になる前に逆引きインデックスを再導入しない。
- 立て直しは採点後・次の照準開始前に行う。GuiMolkkyのstand_up相当の操作で位置/Idを保ち、速度・回転をリセットし、core状態をStandへ戻す。これによりReady時の見た目だけStandにする処理を不要にする。終了時は最終状態を保持できる。これは実際の状態遷移が必要になる操作であり、任意の抽象インターフェースを追加する案ではない。
- coreとの境界：現在のMolkky::aim/hitは&[Skittle]を受け取るので、この案へ署名をそのまま流用しない。推奨はaim/throw/hitを具象GuiMolkkyのメソッドへ置き、既存Molkkyトレイトを採点・勝敗・次手番の共通ルールへ絞る案。Stick/ThrowOutputの関連型も必要性を見直す。CUIは具象側へ物理操作を移す移行が必要になるが、coreのデータ型と採点規則は維持する。関連型や新しいtraitを追加して旧署名を延命する方針にはしない。
- 代替案：GuiSkittleを位置・状態を含む一時的な描画スナップショットに限定すれば、現在のAPIをほぼ維持して描画の組み立てだけを整理できる。ただし物理処理の対応表・力積表は残る。描画と内部処理の両方を簡潔にする今回の目的には、ボディハンドルを持つ実行時モデルを推奨する。
- 移行範囲は区別する。Fieldから個体一覧をGameへ移すことや、描画データの組み立てを整理すること自体は、core APIを変更せず進められる。物理メソッドの再配置と対応表の撤去はその先の設計案であり、GuiSkittle追加の必須条件にはしない。今回のレビューをcore全体の変更許可として扱わない。

### レビュー指摘・確認計画

- 中・app.rs:L34,L41: shrink: 描画のたびにhit・Id照合・Fallの一時コピーを作る処理を、GuiSkittleの状態とボディ位置の読み取りへ置き換える。
- 中・models/field.rs:L7,L33: shrink: Fieldの個体一覧所有と配置生成を分け、GameのVec<GuiSkittle>と配置関数へ移す。
- 中・services/molkky.rs:L10,L11: shrink: 個体の対応表と転倒判定用の別表を、GuiSkittleのbody/core状態へまとめる。物理ワールド自体の構築・更新・衝突判定は引き続き必要であり、型追加だけで全処理が不要になるわけではない。
- [x] 現行の所有・描画時の結合・core APIへの影響を静的に確認する。
- [x] 推奨案と描画スナップショット案を比較し、文書だけの差分を確認する。
- [ ] 実装する場合：coreのId/番号とWorldの対応が保たれ、描画が状態を変更せず、各tickの転倒が停止後も残ることを確認する。
- [ ] 実装する場合：立て直しで位置を変えず、次の照準までにStandへ戻り、採点は一度だけ行われることを確認する。
- [ ] 実装する場合：空Field、標準/別配置、CUI移行、GUI描画・物理・coreルールの回帰を確認する。
- 実装・動作テストは未実行。差分はgit diff --checkのみ確認する。型とAPIの再配置を含む設計案なので、削減行数は未算定。評価対象は所有の一意性、状態の正本、再結合処理の削減とする。

## gui_2d/services/molkky.rsの責務を説明・レビュー（2026-11-04）

- 対象は1d30c0bのgui_2d/services/molkky.rs。core/CUIの同名ファイル、gui_2d/controller.rs・app.rs、対応テストを参照し、GUIの投擲シミュレーションとcoreモデルの橋渡しという意図を説明する。ソース修正は行わない。
- GuiMolkkyはRapierのワールドを所有する方式別の実装であり、Entity/集約そのものではない。coreの採点・勝敗・次手番はMolkkyのデフォルト実装を使い、ScoreBoardへの加算・手番の更新・一度だけの採点はControllerへ任せている。複数オブジェクトを扱うという理由だけで、すべてをサービスへ移す方針ではない。

| メソッド | 現在の責務と配置理由 |
| --- | --- |
| new | PhysicsWorld、形状・質量・減衰・CCDとId対応表を初期化する。Rapierの構築手順と不正配置の拒否を実装側へ閉じ込める。標準番号配列と初期座標はGuiField::initial_positionsが決め、Controllerが渡す。固定の壁もGuiFieldの範囲に合わせて生成し、表示と物理の境界を揃える。 |
| step | 固定の1tickを進め、各個体の接触力積の最大値と停止状態を更新する。描画の頻度と物理の刻みを分け、実時間の待機・イベントループ・採点を含めない。過去の衝突結果を停止後にも読めるよう、最大値だけを保持する。 |
| is_settled | この実装が現在計算中かをControllerへ公開する。初期状態でもtrueであり、ControllerはSimulatingの状態と組み合わせて終了時を判断する。スコア更新や通知の消費は行わない。 |
| pose | coreのIdからRapierのボディを引き、位置(m)と回転(rad)を返す。app/viewがエンジンのハンドルやワールドを直接操作せずに描画できるようにする。ピクセル座標や描画スタイルは決めない。 |
| direction | 投擲位置から目標までの単位方向ベクトルを作る非公開の共通処理。aim/throwで向きの計算を揃え、別の棒・未照準・ゼロ方向等はNoneとする。 |
| aim | 投擲方向の前方・棒とピンの幅に収まる照準候補を読み取る。appが候補の枠を表示するための処理であり、候補外のピンも物理ワールドへ残して連鎖衝突を可能にしている。 |
| throw | 入力を確認して物理ワールドへ初速・向きを設定し、開始の成否を返す。次投準備として位置とIdを維持しつつピンの速度・回転と今回の衝突記録をリセットする。ワールドを更新するため&mut selfを使う。 |
| hit | その投擲で記録した最大力積を閾値と比較し、元のId/番号を持つFallのSkittleを返す。物理更新や点数計算は行わない。描画中と停止時に繰り返し読める蓄積結果であり、新規転倒だけの差分や一度だけ消費するイベントではない。 |

- 同期：リモートの責務整理・固定フィールド/壁反射の変更をfast-forwardで取り込み、最新コードを読み直した。今回の編集差分はこの文書だけ。
- 状態の理由：worldが物理状態の正本、skittle_idsがcoreとの対応、peak_impulsesが停止までの転倒判定材料を担う。2Dの円には立体の横倒しがないため、力積閾値をGUIのシミュレーション上の近似として実装側に置き、公式の採点規則と区別している。
- 関連型Stick/ThrowOutputは、方式固有の棒や戻り値をcoreへ持ち込まずに実装するためのもの。ただしCUIのthrowは確率の読み取り、GUIは投擲開始であり、型の違いを扱えても動作の意味は統一できていない。

### 指摘（優先度付き）

- 中・core/services/molkky.rs:L11 / cui/services/molkky.rs:L27 / gui_2d/services/molkky.rs:L194: shrink: 共通throwの意味を揃えるためだけのAPI維持を再検討する。CUIのthrowは実行経路では使われずgetterと同じ内容。再整理する場合は、共通の採点・勝敗ルールを残し、投擲操作を具象側に置く案、または同じ開始操作に揃える案を比較する。別の抽象トレイトの追加は前提にしない。今回は契約を変更しない。
- 低・gui_2d/services/molkky.rs:L13,L133,L140: shrink: activeは、この実装内で更新しているRapierのsleep状態と二重管理になっている。is_settledでボディのsleepを直接問い合わせれば、同じ状態を同期する処理を減らせる。初期ボディはsleep、成功したthrowはwake、step後は全ボディのsleepを調べる、という現在の更新経路を根拠とする。壁は剛体を持たない静的colliderであり、このbodiesのsleep判定に含まれない。
- activeを取り除く試案だけを/tmpに作りrustfmt後の行数を比較すると8行削減可能。リポジトリのソースへは適用せず、試案の動作テストも行っていない。行数は副次的な情報であり、責務と状態の正本を評価軸にする。
- [x] 現行コード・呼び出し元・テストの期待値を静的に照合する。
- [x] 各メソッドの配置理由と設計上の課題を記録する。
- [x] ソースは変更せず、文書差分をgit diff --checkで確認する。テストは再実行しない。


## GUIの初期配置とゲーム進行を担当箇所へ集約（2026-11-03）

- 対象：直前のレビューで承認された責務整理。標準配置はGuiField、入力から状態更新までの窓口は既存Controller、Dioxusとの接続と描画はappへ置く。core・CUI・物理方式・得点ルール・固定範囲と反射は変更しない。
- 設計：GuiFieldは既存個体のIDに対応する初期配置を生成し、現在位置はGuiMolkkyの物理ワールドだけに保持する。ControllerはGame・GuiMolkky・蓄力・時計を所有し、handle・取消・フレーム進行・投擲完了の記録を担当する。advance_turnは勝敗と次手番の反映であることが分かるupdate_game_stateへ改名する。appのAppStateを削除し、Controllerの状態を描画する。
- API・不変条件：Controller::newで試合を準備し、DOMイベントはController::handleへ渡す。照準解除はController内部の入力終了処理へ集約し、aimは候補の問い合わせ、throwは発射開始、hitは転倒結果の取得を維持する。取消で別pointerや投擲中の状態を壊さず、停止後の採点は一度だけ行い、勝利の反映で投擲履歴を追加しない。
- テスト方針：動作を変えないリファクタリングとして、既存テストを変更前後で確認する。配置生成の新しい呼び出し口はテストを先に書く。進行テストをtests/gui_2d/controller.rsへ移し、appのテストは描画・接続対象に絞る。検証内容を削らず、登録用ファイルにテスト本体を置かない。
- [x] 変更前のGUIを含むRust全体を確認する。
- [x] GuiFieldの初期配置APIをテストし、標準配置の生成を移す。
- [x] Controllerへ入力・取消・進行・結果反映を集約し、AppStateを削除する。
- [x] 対応する場所へテストを移し、既存の期待値を保って成功させる。
- [x] Rust全体・feature分離・fmt・Clippy・Webビルドとブラウザー操作を確認する。
- [x] READMEと記録を更新し、差分を確認する。

### 実行・レビュー結果

- 変更前：`cd rust_molkky && cargo test --locked --features gui_2d --quiet`が173件成功。ログは`/tmp/molkky-responsibility-before.log`。
- Red：GuiField::initial_positionsとTHROW_POSITIONを使うテストを先に追加し、未実装APIによるE0599を確認した。実行時の振る舞いの失敗とは区別する。ログは`/tmp/molkky-responsibility-red.log`。
- Green：標準の四列・各座標・投擲位置から1mの距離・個体IDの保持・再生成時の同一結果を確認した。モデルからservicesへ依存せず、初期配置はControllerが渡す半径を使って計算する。現在位置の保存先は物理ワールドだけを維持した。ログは`/tmp/molkky-responsibility-field.log`。
- Refactor：AppStateを削除し、app.rsは296行から113行になった。ControllerがGame・GuiMolkky・蓄力と時計を所有し、handle・update・finish_throw・update_game_stateを担当する。reset_inputで蓄力・照準・速度を解除し、取消の際のReadyへの遷移もController内へ集約した。Gameはgame()で読み取り専用に参照する。物理・coreのルール・CUI・依存関係は変更していない。
- API整理：Controller::default()と公開されていた入力補助メソッドを、Controller::new(teams)と内部のイベント処理へ整理した。pressは自身のGamePhaseを参照し、preview・releaseも自身のGuiStickを使うため、所有する状態を引数で渡し直す必要がなくなった。配置移動中の借用エラーもこの整理で解消したが、Red確認としては数えない。
- テストの移動：入力・取消・時刻・物理停止後の一度だけの採点・交代・終了はtests/gui_2d/controller.rsの18件へまとめ、controller.rsのcfg(test)から登録する。appの4件は初期描画・照準と速度・勝利/勝者なしの表示・固定viewBoxを扱う。旧appの配置確認はFieldの座標テストへ、勝利や照準の表示確認はappの描画テストへ移し、検証対象と期待値を維持した。登録用のtests/gui_2d.rsから重複するController登録を外した。
- 取消の追加確認：所有pointerの取消後に、照準なし・速度0・入力所有なし・Readyとなり、棒のIDを保ち、続く解放で投擲を記録しないことを確認した。このテストは追加時点で成功しており、Red確認済みとは扱わない。
- Rust：`cargo test --locked --features gui_2d --quiet`が177件（Controller18＋app4＋application81＋core_models49＋gui_2d25）、既定の`cargo test --locked --quiet`が135件、`cargo test --locked --no-default-features --features gui_2d --quiet`が96件成功。失敗・無視0件。ログは`/tmp/molkky-responsibility-{green,default,gui-only}.log`。
- 整形・Lint・ビルド：`cargo fmt --check`、`cargo clippy --locked --all-targets --features gui_2d -- -D warnings`、`cargo build --locked --no-default-features --features gui_2d_web --bin gui_2d --target wasm32-unknown-unknown`が成功。ログは`/tmp/molkky-responsibility-clippy.log`と`/tmp/molkky-responsibility-web-build.log`。
- Web：ユーザーが起動した既存の8080番サーバーを使い、`NODE_PATH=/tmp/molkky-browser-tools/node_modules CHROMIUM_PATH=/usr/bin/chromium node tests/gui_2d/controller.js`が成功。capture・取消・リサイズ・領域外解放・採点・交代・失格・勝利表示・終了後入力無効化・固定範囲・壁での反射を確認した。pageerrorとalertは0件。サーバーの追加起動・停止は行っていない。ログは`/tmp/molkky-responsibility-browser.log`。
- 最終確認：READMEの責務・API・テスト実行先と件数を更新し、git diff --check成功。デスクトップ版の実操作は今回は未確認。関連する差分をConventional Commitで記録し、通常pushする。

## app.rsの責務配置を再レビュー（2026-11-03）

- 対象・入力：ユーザー指定のAppStateのnew・handle・cancel_charge・reset_aim・update・finish_throw・advance_turn。app.rs、controller.rs、models/{field,game,stick}.rs、GuiMolkky、coreのMolkky、CUIのrunと対応するテストを読む。1人がブラウザーで操作する既存GUIを前提とし、今回は説明とレビュー記録のみでコードを変更しない。
- [x] 各メソッドの呼び出し元と、参照・更新する状態を確認する。
- [x] appへ配置した当時の理由と、現在の責務として適切かを区別する。
- [x] aim・throwへ統合した場合のAPIと動作への影響を確認する。
- [x] 優先度・根拠・配置案・未確認事項を記録し、文書差分を確認する。

### 配置した理由と判断

- 当時の実装ではControllerを入力の所有・蓄力の解釈、Gameを状態保持に限定し、AppStateが両者とGuiMolkkyを調整する役割を持った。そのため、入力結果の反映、物理の進行、採点・交代がappへ集まった。この分担は一つの設計判断であり、Dioxusやアニメーションがapp.rsへの配置を必須にするものではない。前回の説明は、処理を分ける必要性をapp.rsへ置く理由として扱っており、配置の検討が不足していた。

| メソッド・根拠 | appへ置いた理由 | レビュー結果・配置案 |
| --- | --- | --- |
| new：app.rs:27、field.rs:10 | Game・GuiMolkky・Controllerを組み立てる時に、物理ワールドへ渡す個体IDと初期座標も生成した | 依存オブジェクトの組み立てと標準配置の生成が混在している。番号配列・間隔・初期座標の生成はGuiFieldへ集約する。new全体をFieldへ移すとチーム・入力・時計の生成までFieldの責務になるため、生成する対象を分ける。現在位置は既存の物理ワールドが保持し、重複して保存しない |
| handle：app.rs:65、controller.rs:70 | 入力解釈はController、Gameと物理状態への適用はAppStateという分担にした | イベントの振り分け・入力変換と、それに伴うゲーム操作はControllerへまとめるのが自然。Dioxusのonmounted・JSとの接続・Signalへの通知はappに残す |
| cancel_charge：app.rs:129、controller.rs:146 | Controllerのchargeを消した後、別の所有者であるAppStateがGuiStickの照準・速度とGamePhaseを戻す必要があった | 取消の操作が二か所に分かれている。Controllerの取消操作で、ポインター所有・照準・速度・Charging→Readyを一緒に扱う。投擲中のBlurと別pointerの取消でゲームを戻さない既存の条件は保つ |
| reset_aim：app.rs:136、services/molkky.rs:173 | 取消・無効な投擲・投擲完了から、同じGuiStickのIDを保って照準と速度を消す処理を共有した | 入力状態の解除である。Controllerの入力終了処理へまとめるか、GuiStickの値を初期状態へ戻す操作として呼ぶ。現在のaimは&selfで候補を取得する問い合わせで、renderからも呼ぶ。照準を消す処理をこのaimへ混ぜると問い合わせと状態変更の意味が衝突する |
| update：app.rs:140 | Frame通知を受けて、蓄力の表示値と経過時間を反映し、固定刻みでstepして停止後に採点を依頼するため | ゲームのフレーム進行を担当するControllerへ置ける。Rapierの1tickはGuiMolkkyのstepが担当する。アニメーションのため開始と継続更新は分ける必要があるが、継続更新をapp.rsへ置く必然性はない |
| finish_throw：app.rs:163、services/molkky.rs:194 | throwは発射開始の成否boolを返すため、後のフレームで停止したときだけ採点・記録・照準解除・交代を行う処理を別にした | 1投の結果をGameへ適用する進行処理なのでControllerへまとめられる。現在のthrowはstickだけを受け、teamsや得点表を扱わない。throwを1投の開始から完了まで扱うAPIへ再設計するなら統合は可能だが、同期的に停止まで計算すると途中描画が止まる。既存APIのままなら開始と完了処理を分ける |
| advance_turn：app.rs:180、core/services/molkky.rs:25 | 初期化時と投擲完了時に、判定結果をチーム・手番・GamePhaseへ反映するため | judge_gameで勝者を確認し、勝者がいなければnext_teamで次手番を選ぶ。全チームのturnを更新し、勝者へwith_winで50点を反映し、次手番があればReady、なければFinishedにする。単なる手番送りより広い責務であり、「勝敗・次手番を反映する」ことが分かる名前とControllerへの配置が適切。勝敗ルールそのものは既存coreサービスへ委譲し続ける |

### 指摘と最小の整理方針

1. 優先度中・標準配置の生成が画面組み立てへ混在：番号配列や配置間隔を変える際にapp.rsの三角格子計算まで編集する必要がある（app.rs:32）。GuiFieldに個体集合があるため、初期配置の生成をそこへ集約し、appのnewは生成結果をGuiMolkkyへ渡す処理にする。
2. 優先度中・入力処理と状態遷移の所有が分かれている：Controller::cancelは入力のchargeだけを消し、対応する照準・速度・GamePhaseの更新にはAppState::cancel_chargeも必要になる（controller.rs:146、app.rs:113）。既存Controllerをゲーム操作の窓口として揃え、handle・取消・フレーム進行・完了結果の適用をそこへまとめる。appの責務はDioxusとの接続と描画にする。
3. 優先度中・advance_turnの名称から終了処理が読めない：相手の3回目のミスで、同じメソッドが勝者の50点化とFinishedへの遷移まで行う（app.rs:180、tests/gui_2d/app.rsのscoring）。上記の配置整理と合わせて、勝敗と次手番を反映する処理として名前を付ける。

- 判断：上記は保守性・責務配置の指摘であり、現状の通常操作で動作不良が再現したという判定ではない。優先するのは1・2。単に全処理をaim・throwへ押し込むと、候補の問い合わせと入力状態の変更、物理の発射と試合全体の進行が混在する。新しい抽象層・trait・管理オブジェクトは提案せず、既存Field・Controller・GuiMolkkyの担当を揃える。
- 検証：対応テストから取消・投擲停止後の一度だけの採点・交代・終了の期待値を確認した。コード変更なしの静的レビューのため、Rustテスト・ブラウザー操作・Red/Green/Refactorは今回実行していない。文書のみgit diff --checkで確認する。配置案の実装・行数削減量は未検証。
- 対応状況：このレビューで提案した配置の整理は、直上の「GUIの初期配置とゲーム進行を担当箇所へ集約」で実装・検証した。

## gui_2dの表示範囲を固定し外周で反射させる（2026-11-03）

- 対象：GUIのフィールド範囲、GuiMolkkyの物理ワールド、appの描画、対応するRust・DOMテストとREADME。coreのルール・CUI・traitのAPIは変更しない。
- 仕様・判断：現在の初期範囲を固定する（x=-0.3〜0.3m、y=-0.15〜1.35m）。GuiFieldに範囲と外周の壁の厚さ2cmを定義し、表示と物理境界で共用する。四辺にRapierの静的な壁を置き、棒とスキットルを反射させる。反発係数は0.5とし、既存の減衰と停止・採点・交代を維持する。壁・厚さ・反発係数はこのGUIの取り決めである。
- レビュー・優先度高：appのrenderは毎フレーム全物体の座標でviewBoxを拡張しており、物体が飛び出すたび画面が縮小する。動的な範囲計算を削除する。
- 責務のレビュー：throwには立て直し・投擲方向と初速の設定、aimには照準候補の選択、hitには蓄積した力積による転倒結果の取得が既にまとまっている。stepはフレームごとの物理更新と接触履歴の保存、is_settledは採点時機の確認、poseは描画用の座標取得に必要。throwで停止まで計算すると途中の動きを描画できず、hitに更新を移すと描画からの問い合わせで物理状態が変わる。directionはaim・throwの共通処理なので、複製せず維持する。
- [x] 変更前のGUIを含む全Rustテストを確認する。
- [x] 投擲中・停止後もviewBoxが変わらないテストのRedを確認する。
- [x] 四辺・角・高速投擲で反射して範囲内に残り、最終的に停止するテストのRedを確認する。
- [x] 表示と物理で同じ固定範囲を使い、既存の採点・再投擲・個体IDを維持する。
- [x] 全Rustテスト・整形・Clippyとブラウザー操作を確認し、結果を記録する。

### 実行・レビュー結果

- 変更前：`cd rust_molkky && cargo test --locked --features gui_2d --quiet`が168件成功。ログは`/tmp/molkky-fixed-field-before.log`。
- Red：appの固定範囲テストは「表示範囲が変わりました」で失敗。物理側の4件は棒またはスキットルが外へ出るため失敗した。ログは`/tmp/molkky-fixed-field-red.log`と`/tmp/molkky-wall-red.log`。同じ境界を描画と衝突で共有する変更なので、表示1件と物理4件を一組の退行テストとして先に追加した。
- Green：動的なviewBox計算を削除し、外周を描画して同じ内側の面に静的コライダーを置いた。棒・スキットルのCCDを有効にし、四辺・角・120m/sの投擲、強く打たれたスキットルの保持と停止を確認した。棒の回転を含む全体形状を各更新で調べ、描画範囲からのはみ出しは物理計算の誤差5mm以内で検証した。独自の反射式、接触精度の追加設定、更新周期の変更は加えていない。
- 既存テストの前提変更：反射があると、後方へ強く投げても戻って当たる。ミスの検証は届かない弱い投擲へ変更し、0点・一度だけの採点・交代・履歴維持の期待値は保った。初期配置のfixtureは固定範囲内へ移し、高速衝突での個体識別は最初の衝突時に確認する。後続の反射で別個体に当たることを誤って失敗としない。
- レビュー・優先度中：静的な壁のコライダーには親RigidBodyがない。stepの停止中接触の判定でparentをunwrapせず、壁は動かないものとして扱う。壁を停止判定の対象ボディへ追加せず、投擲が終わらなくなる問題も避けた。
- Refactor：触れたapp・GuiMolkky内のpinという変数名をskittleへ統一した。throw・aim・hitは既存の責務を維持し、step・is_settled・pose・共有するdirectionを無理に統合していない。新しいtrait・依存関係・物理の抽象層は追加していない。
- Rust：`cargo test --locked --features gui_2d --quiet`が173件（app11＋application81＋core_models49＋gui_2d32）成功、失敗・無視0件。`cargo fmt --check`、`cargo clippy --locked --all-targets --features gui_2d -- -D warnings`も成功。ログは`/tmp/molkky-fixed-field-green.log`と`/tmp/molkky-fixed-field-clippy.log`。
- Web：起動済みdxがWeb版を再ビルドした。`NODE_PATH=/tmp/molkky-browser-tools/node_modules CHROMIUM_PATH=/usr/bin/chromium node tests/gui_2d/controller.js`が成功。従来のcapture・取消・採点・失格・勝利に加えて、各状態でのviewBox固定と、横向き投擲中の座標をフレームごとに採取して反射・範囲内への保持・停止後の交代を確認した。pageerrorとalertは0件。ログは`/tmp/molkky-fixed-field-browser.log`。
- 検証範囲：JSの構文確認と`git diff --check`成功。デスクトップ版の実操作は今回再確認していない。ローカルWebサーバーは`http://127.0.0.1:8080`で継続して起動している。

## ローカル環境でgui_2dを起動（2026-11-03）

- 対象：既存のGUI起動手順とローカルの実行環境。ゲームの仕様・ドメインAPIの変更は予定しない。
- 期待する結果：GUIを起動し、描画と投擲・採点・交代を確認して、再起動コマンドを残す。
- 環境：Dioxus CLI 0.7.9とChromiumは導入済み。デスクトップ用のlibxdo等が未導入のため、Web版を使う。必要なwasm32ターゲットを追加する。
- [x] README・Cargo feature・GUIの入口・実行環境を確認する。
- [x] Web版をビルドしてローカルサーバーを起動する。
- [x] DOMテストのSVG選択をゲーム領域内に限定し、dxの開発用表示があっても実行できるようにする。
- [x] 実際のブラウザーで描画と操作を確認する。
- [x] 起動手順と検証結果を記録し、差分を確認する。

### 実行・レビュー結果

- 起動：`rustup target add wasm32-unknown-unknown`の後、`cd rust_molkky && dx serve --web --bin gui_2d --no-default-features --features gui_2d_web --locked --addr 127.0.0.1 --port 8080 --open false --interactive false`でビルド成功。dxがwasm-bindgen-cli 0.2.129とesbuildを準備した。`http://127.0.0.1:8080`で配信し、`xdg-open`で通常のブラウザーも開いた。サーバーは起動したままにする。
- レビュー・優先度中：既存DOMテストはページ全体からSVGを選ぶため、dxの開発用表示にあるSVGとゲームのSVGが一致し、strict mode violationで停止した。`tests/gui_2d/controller.js`の選択範囲を既存のゲーム領域`#molkky-input`内に限定した。検証内容や期待値は変更していない。
- Red：変更前の`NODE_PATH=/tmp/molkky-browser-tools/node_modules CHROMIUM_PATH=/usr/bin/chromium node tests/gui_2d/controller.js`が、上記のSVG複数一致で失敗した。環境準備の不足や中断したdesktopビルドはRedに数えない。
- Green：同じコマンドが成功。12個体の描画、capture・取消・リサイズ・メーター・領域外解放、採点・交代、連続ミスでの失格、最後の生存者の勝利、終了後の入力無効化を確認した。pageerrorとアプリのalertは0件。
- 描画：別のChromiumページで初期状態がready、スキットル12本、pageerrorなしを確認。`/tmp/molkky-gui-running.png`でフィールド・棒・得点表を目視確認した。
- Refactor：追加なし。READMEにdxによる起動手順を追記し、既存のPython起動スクリプトとの使い分けを明記した。Rustソース・Cargo設定・lockは変更していない。
- 検証：DOMテスト、`node --check rust_molkky/tests/gui_2d/controller.js`、`git diff --check`を実行。Rustソースの変更がないためcargo testは今回再実行していない。デスクトップ版の起動は未確認。

## gui_2dのapp・DOM接続・起動を実装（今回の一段階）

- 今回は明示指定されたapp・DOMイベント接続・GUI起動を一つの接続段階として実装する。coreの採点・勝敗・手番と既存controller/view/GuiMolkkyを使い、追加のルールや物理方式は作らない。
- AppStateはGame/Controller/GuiMolkkyを一つずつ保持し、Ready→Charging→Simulating→Ready/Finishedを管理する。固定刻みの更新を積算し、停止時だけ一度採点・得点表更新・勝者/次手番の判定を行う。タブ復帰時は1フレーム250msまで物理時間を進める。
- 標準の番号配列は手前から1/2、3/10/4、5/11/12/6、7/9/8。間隔は物理の半径に合わせる。見下ろし操作用の初期投擲距離は1mとし、公式競技の距離再現とは区別する。全チームで同じField/Stickを使い、移動後の位置とIdを保持する。
- DOM側は標準Pointer EventsとrequestAnimationFrameを使う。Webとdesktop WebViewで同じJSをdocument::evalで接続し、左マウスの同期的なcapture、解放、取消、focus喪失を扱う。入力解釈と座標変換はRust controllerへ渡し、ゲーム進行をJSへ複製しない。evalはアンマウント時に停止させる。
- CargoはCUIを既定featureにし、GUI Web/desktopを選択する。CUIの既定起動方法を維持し、WebではCUIの端末/乱数依存を除外する。UUIDはWebでjsを有効にする。新たなgui_2dバイナリを追加する。

### テストリスト

- [x] 変更前GUI featureあり158件成功。リモートとの差分なし。
- [x] 初期配置・最初の手番・描画部品の合成を確認する。
- [x] 押下・照準・メーター・解放・取消を接続し、投擲の成否で状態を更新する。
- [x] 物理停止後の一度だけの採点と手番交代、勝利・超過・失格・勝者なし終了を確認する。
- [x] Web/desktopの起動設定とビルドを確認する。実行できない環境の制約は区別して記録する。
- [x] ブラウザーで実DOMのcapture・領域外解放・focus喪失・得点更新・結果表示を確認する。
- [x] 既定CUI、feature分離、全テスト・fmt・Clippy・差分を確認しcommit・通常pushする。

### 実行・レビュー結果

- Red：app.rsにテスト登録だけを置き、tests/gui_2d/app.rsのテストから未実装AppState/DomEventを参照して失敗を確認した（/tmp/gui-app-red.log）。テスト本体はtests以下に置き、物理結果を受け取るprivateな境界をテストするためapp.rsのcfg(test)から登録している。
- Green：appの10件で、標準配置・初期手番・部品合成、取消と無効な投擲、非有限/負の時刻、実際の物理停止待ちと一度だけの採点、タブ復帰の更新上限、複数本採点、超過25点、50点、連続ミス失格、最後の生存者、単独失格を確認した。採点境界のテストでは転倒結果を固定し、物理方式の差し替え用traitは追加していない。
- Web：wasm32ターゲットを導入し、cargo build --locked --no-default-features --features gui_2d_web --bin gui_2d --target wasm32-unknown-unknownが成功。Dioxus最小構成ではweb_sys::Window::locationが未定義だったため、同じweb-sysのLocation featureだけを有効にした。Webビルド用の準備エラーであり、アプリのRedには数えない。wasm-bindgen 0.2.129で生成したWeb版を起動した。
- DOM：tests/gui_2d/controller.jsをChromiumで実行し成功。12個体、右ボタン無視、capture設定、blur取消、lost capture取消、リサイズ後の照準、メーター、領域外解放、5回のミスによる失格/最後の生存者の勝利、履歴、終了後の入力無効化を実際のDOMで確認した。pageerrorとアプリのalertは0件。capture喪失のテストは、pending captureを実際に成立させてから解除し、次のpointerイベントでlostpointercaptureを配送する手順へ修正した。
- desktop：最初はGTK/WebKitの開発ライブラリ不足でビルドできなかった。Debianパッケージを/workspace/.cloud-tools/desktopへ展開し、pkg-configの参照先と開発用symlinkを整えてビルド成功。Xvfbで起動し、描画・タイトル・蓄力・領域外解放・採点・交代を確認した。WebKitの固定helperパスは検証用prootで対応し、headless環境ではWebKit sandboxを無効にした。これはローカルの検証環境だけの設定で、起動スクリプトやアプリの設定には入れていない。Windows/macOSは未実行。
- 構成：CUIはdefault featureとして維持し、既存81件は引き続き既定テストで実行する。GUI-onlyではCUIをビルドしないため、そのCLIテストターゲットにrequired-featuresを指定した。viewのテストfixtureもCUI依存を取り除いた。Cargo.lockの既存バージョンは削除・更新せず、主にdesktopレンダラーの依存433件を追加した。CSSはview/style.cssへまとめ、JSはDOMと時計の通知だけとし、採点・勝敗・物理計算を持たせない。
- 最終確認：cargo test --locked --features gui_2dは168件（app10+既存81+49+28）、既定は134件、--no-default-features --features gui_2dは87件が成功、失敗・無視0件。cargo fmt --check、cargo clippy --locked --all-targets --features gui_2d -- -D warnings、JS/シェルの構文確認、git diff --checkも成功。CUIはcargo run --lockedへqを入力して既定の入口と正常終了を確認した。
- 照準の接続：Appの照準候補表示のテストを先に追加し、class=aimedがない失敗を確認した（/tmp/molkky-aim-red.log）。GuiMolkky::aimを呼び、候補の枠を強調する表示を接続して成功した。候補外も物理対象に含める仕様は維持する。
- 成功ログ：Rustの全体は/tmp/molkky-final-tests.log、/tmp/molkky-default-tests.log、/tmp/molkky-gui-only-tests.log。Web/desktopのビルドは/tmp/molkky-web-build.log、/tmp/molkky-desktop-build.log。実DOMの最終結果は/tmp/molkky-browser-final.log。Linuxでは/tmp/molkky-desktop-scored.pngで採点後の交代も目視確認した。
- 残る確認範囲：WebはChromium、desktopはLinuxの仮想画面での検証。タッチ入力、チーム設定画面、再試合、公式競技距離の再現は今回の範囲に含めていない。指定された接続・起動の段階を終えたらcommit・通常pushして停止する。


## gui_2dのcontrollerを実装（今回の一段階）

- 残っているcontroller/app/起動のうち、今回はcontrollerの入力解釈だけを実装して停止する。先に全段階を実装しないという合意を維持する。追加依存やcore/models/view/servicesの変更は行わない。
- screen_to_worldはSVGの中央寄せ・アスペクト比維持に合わせ、描画領域のCSSピクセル矩形とworldの[x,y,幅,高さ]からY上向きの座標へ変換する。余白や領域外、非有限値、正でない寸法はNone。矩形はborderを除いた描画領域とする。
- Controllerは押下中のpointer_id・開始時刻・最後の有効な目標だけを保持する。Ready時の左ボタンだけを受け付け、別ポインターや重複押下、計算中・終了後の押下は無視する。phaseは参照するだけで、状態遷移はappが行う。
- 速度は経過時間の三角波（0〜10m/s、往復2秒）とし、フレーム数に依存しない。previewとreleaseは同じ計算を使って同じcore::StickのIdを持つ新しいGuiStickを返す。開始前の時刻は経過0として扱う。
- 領域外の移動は最後の有効な照準を保持する。解放は座標変換に依存せず、同じpointerの左ボタン解放で一度だけGuiStickを返す。pointer cancel/lost captureでは該当pointerだけ、focus喪失では全入力を破棄する。
- この段階はDOMに触れない。後続appがpointer captureを実際に設定・解除し、pointer/blur等のイベントをcontrollerへ渡す。実時間時計、描画ループ、物理・採点・状態遷移・Web/desktop起動は今回作らない。

### テストリスト

- [x] 変更前GUI featureあり150件成功。リモートとの差分なし。
- [x] 画面の位置・拡縮・Y反転と縦横の余白を考慮して座標変換できる。
- [x] 不正な座標・寸法と領域外を拒否する。
- [x] Ready時の左ボタンだけ開始し、別pointerや再押下では上書きしない。
- [x] 時間ベースで速度が上下し、表示値と解放時の初速が一致する。
- [x] 有効な移動だけ目標を更新し、領域外でも同じpointerの解放は処理する。
- [x] 解放は一度だけ。取消・focus喪失は投擲せず入力を破棄する。
- [x] Red/Green、featureあり/なしの全テスト・build・fmt・Clippy・差分を確認する。
- [x] controllerの段階だけをcommit・通常pushして停止する。

### 実行・レビュー結果

- Red：tests/gui_2d/controller.rsのテスト8件を先に登録し、cargo test --locked --features gui_2d --test gui_2d controllerで未実装controllerのE0432を確認した（/tmp/gui-controller-red.log）。
- Green：同じコマンドで8件成功。中心・角・余白・範囲外・不正寸法の座標変換、左ボタンとpointerの所有、経過時間の速度、移動と領域外解放、解放の重複防止、個別取消とfocus喪失を確認した。
- Refactor：テスト名のReadyを日本語へ変更しnon_snake_case警告を解消した。ControllerはChargeのOptionだけを持ち、開始フラグとpointer状態の二重管理やイベント用traitは作らない。開始時刻から算出し、フレームごとに速度を増減して誤差を積み上げる実装は使わない。
- 全体：cargo test --locked --features gui_2d --quietは158件（81+49+28）、featureなしは134件成功、失敗・無視0件。cargo build --lockedはfeatureあり/なしとも成功。cargo fmt --check、cargo clippy --locked --all-targets --features gui_2d -- -D warnings、git diff --checkも成功した。
- 境界の確認：追加依存なし。Cargo.toml/lock、core/CUI、既存models/view/servicesに差分がない。ControllerはDOMや時計の取得、物理・採点・勝者判定を行わず、GamePhaseを変更しない。各入力から新しいGuiStickを作り、渡されたcore::StickのIdを保つ。
- 残り：DOMイベント接続・pointer capture・実時間の更新・標準配置・appでの一度だけの採点・GUI起動は次段階。今回の検証は入力解釈のテストであり、実画面での操作は行っていない。


## gui_2dのservicesへRapierを統合（今回の一段階）

- 対象はgui_2d/services/molkky.rsのGuiMolkky。physics.rsは作らず、Rapier 0.36.1のPhysicsWorldを使う。app/controller、GUI起動、実時間のループ、得点表や手番の更新は今回実装しない。
- aimは投擲方向の直線上にある候補を選ぶ。候補外のピンも物理ワールドに残し、連鎖衝突を扱う。throwは目標とm/sの初速を使って棒へ速度を設定する。stepは1/60秒進め、個体ごとの接触力積の最大値を記録し、hitは閾値以上の個体だけをFallとして返す。Idと番号は再生成しない。
- 物理の所有はGuiMolkkyだけ。coreのStick/Skittleの複製リストやTeamは保持せず、ボディとIdの対応、初期投擲位置、今回の衝突記録、計算中かを保持する。位置と回転は読み取り用poseでviewへ渡せる。通常12本の小さな対応表なので描画用の逆引きは走査する。
- coreの最小API変更：throwを&mut selfにして関連型ThrowOutputを追加する。CUIはf64の確率を維持し、GUIは開始できたかのboolを返す。RefCellや同名の別throwを追加しない。CUIのhitは棒の確率を直接参照して既存動作を保つ。dyn Molkkyの関連型指定と既存throwテストの可変束縛も移行する。採点・勝者・次手番の実装は変更しない。
- ゼロ重力、円のスキットル、Y軸に沿ったカプセルの棒、線形/角減衰、棒のCCDを採用する。寸法はm、質量はkg、力積はN·s、角度はラジアン。値はGuiMolkkyの関連定数へまとめる。高さ方向の転倒を接触力積で近似するもので、実際の横倒しの再現とは区別する。
- 1回のthrowから停止まで、再投擲は受け付けない。未照準・同一点・非有限値・正でない速度・別の棒は状態を変えずfalse。次のthrowで位置を維持したままピンを立て直し、速度・角速度・今回の記録を消す。停止は全ボディのsleepで判定する。終了通知の一度だけの採点は後続appの責務。

### テストリスト

- [x] 変更前GUI featureあり142件成功。リモートとの差分なし。
- [x] 照準候補と物理対象を区別し、初速・向き・1tickの移動を確認する。
- [x] 無効な投擲と計算中の再投擲で状態を変えない。
- [x] 衝突の強弱、高速投擲、連鎖衝突、ミス、Id保持と重複しない転倒結果を確認する。
- [x] 停止・再投擲・位置の維持・前回記録の消去を確認する。
- [x] coreの採点・勝者・次手番のデフォルト実装がGUIから利用できる。
- [x] Red/Green、featureあり/なしの全テスト・build・fmt・Clippy・差分を確認する。
- [x] servicesの段階だけをcommit・通常pushして停止する。

### 実行・レビュー結果

- Red：servicesのテストを先に登録し、cargo test --features gui_2d --test gui_2dで未実装servicesによるE0432を確認した（/tmp/gui-services-red.log）。依存追加でlock更新後は--lockedを使用した。
- Green：サービス8件が成功。初期の弱速テストは減衰後の最大移動距離より遠くにピンを置いていたため、実際に接触する配置へ修正した。連鎖テストは直接狙う候補が1本で、候補外の2本目が衝突経由で倒れる配置へ修正して確認した。転倒閾値をテストへ合わせる変更は行っていない。
- 結果：GUI featureありの全150件（81+49+20）、なしの134件が成功、失敗・無視0件。featureあり/なしのcargo build --locked、cargo fmt --check、git diff --checkも成功した。
- Clippy：cargo clippy --locked --all-targets --features gui_2d -- -D warningsが成功。API変更で触れたcore/services/molkky.rsの既存needless_returnも末尾式へ置き換えて解消し、lintの除外は使用していない。
- API移行：Molkky::throwは&mut selfとThrowOutputへ変更した。既存のdyn指定とCUIのthrowテストだけを移行し、CUIの確率やcoreの採点・勝敗・手番の期待値は維持した。外部実装も関連型指定と可変レシーバーへの移行が必要。READMEへ説明した。
- レビュー：PhysicsWorldへ任せて独自パイプライン・イベントハンドラー・同期プリミティブは作らない。stepは接触力積の統計を保存し、hitが閾値とcoreのStand/Fallを使って転倒結果を構成する。constructorでId重複・非有限配置が物理エンジンへ入るのを防ぎ、throwは無効な入力を変更前に拒否する。未知のIdのposeはNone。別の棒のhitでは結果を返さない。
- 制約：物理の設定は近似用の値であり、実際の高さ方向の転倒や実機の挙動を保証するものではない。初期配置・入力・描画ループ・投擲終了後の一度だけの採点・GUI起動はまだ実装していない。既存models/viewは変更せず、servicesの段階で停止する。


## gui_2dのviewを実装（今回の一段階）

- 対象はview/{field,skittle,stick,speedometer,score,result}.rsのみ。app/controller/GuiMolkky、Rapier、GUI起動、入力イベントは実装せず、この段階で停止する。coreと既存modelsは変更しない。
- Dioxus 0.7.10のminimalを任意feature gui_2dで有効にする。Web/desktopランタイムやOS依存は今回導入しない。HTML/SVGの検証には公式のdioxus-ssr::render_elementをdev-dependencyとして使う。
- viewはElementを返す関数で構成し、coreを読み取り、個体やチームを重複所有しない。フィールドは呼び出し側の範囲とchildrenを描画し、位置・回転は引数で受け取る。標準配置や物理更新は決めない。SVGはworldのY上向きを反転し、番号の文字は読みやすい向きへ戻す。
- 棒は位置・回転（ラジアン）・長さ・幅で丸い端の棒を描く。スキットルは番号とStand/Fallを描き分ける。寸法・入力値の検証や画面からworldへの入力座標変換は呼び出し側の責務とする。
- 速度メーターはnativeのmeterとm/s表示を使い、速度の時間変化や入力処理は持たない。得点表はTeam/ScoreBoardの既存total/history/state/turnを表示する。勝者は呼び出し側のOption<&Team>を受け取り、viewでjudge_gameを複製しない。

### テストリスト

- [x] 変更前cargo test --locked --quiet：134件成功、失敗・無視0件。リモートとの差分なし。
- [x] SVGのフィールド範囲、Y方向、childrenの合成を確認する。
- [x] スキットルの番号と転倒状態、棒の位置・回転・寸法を確認する。
- [x] 速度値・上限とm/s表示を確認する。
- [x] 得点表のチーム名、現在手番、履歴・合計・計算状態と文字のエスケープを確認する。
- [x] 渡された勝者と勝者なしの結果表示を確認する。
- [x] 未実装viewのRed→Green、featureあり/なしの全テスト・build・fmt・Clippy・差分を確認する。
- [x] viewだけをcommit・通常pushし、次段階へ進まず提示する。

### 実行・レビュー結果

- Red：tests/gui_2d/view/のテストを先に追加し、cargo test --features gui_2d --test gui_2dで未実装viewのE0432を確認した（/tmp/gui-views-red.log）。依存導入に伴うCargo.lock更新後は--lockedで検証した。
- 実装途中：SVGのaria_labelはDioxusのSVG属性として未定義だったため、標準のaria-label属性へ修正した。また、SSRは名前の特殊文字を数値文字参照へエスケープする。初期テストが名前付き文字参照だけを期待していたため、HTMLとして有効な両表現を許可した。描画側のエスケープは変更せず、テストの準備上の問題として扱う。
- Green：描画テスト8件と既存models4件が成功。フィールドのviewBox/Y反転と子要素、転倒状態、棒の角度・寸法、meterの速度・上限、名前のエスケープ、得点表の履歴・合計・状態、勝者/勝者なしを確認した。現在手番はaria-currentと可視の▶で示す。
- 全体：cargo test --locked --quietは134件、--features gui_2d付きは142件（81+49+12）が成功、失敗・無視0件。cargo build --lockedはfeatureあり/なしの両方が成功。cargo fmt --check、git diff --checkも成功した。
- Clippy：--all-targets --features gui_2d -- -D warningsは既存core/services/molkky.rs:37のneedless_returnで停止する。同lintだけをCLIの-A clippy::needless_returnで除外した確認は成功した。core修正やソースへのallow追加は行わない。
- 依存の境界：cargo tree --locked -e normal --depth 1で既定の実行時依存にDioxusが含まれないことを確認した。dioxus-ssrはテスト用の依存。Cargo.lockの既存依存の表記変更は同名パッケージの別バージョン追加による識別子の明確化であり、既存パッケージの更新は行っていない。
- レビュー：viewはElement生成だけを行い、core/modelsやCUIに差分がない。入力イベント、状態更新、タイマー、標準配置、物理、勝利/採点の再実装は含めない。HTML/SVG出力の検証であり、ブラウザー・デスクトップでの見た目と実操作は未確認。GUI用ランタイムと起動入口は今回追加せず、次段階へは進まない。


## gui_2dのmodelsを先に実装（2026-11-02）

- 今回の範囲はmodelsのみ。後続は入力・表示、GuiMolkkyへのRapier統合、coreとの接続へ段階的に進める。physics.rsは作らず、各段階の差分を提示して停止する。
- GuiFieldはcoreのFieldとVec<Skittle>を合成し、全チームで共有する。通常は1〜12番を立った状態で作るが、既存のFieldと個体集合も受け取れる。番号ではなくIdで個体を保持する。
- GuiStickはcoreのStick、目標位置Option<[f32; 2]>（フィールド内のメートル座標）、初速f32（m/s）を保持する。未照準はNone、初速は0。初速を確率として制限しない。入力の検証・変換は後続controllerで扱う。
- GameはVec<Team>、共通のGuiField/GuiStick、GamePhaseを所有する。Readyから開始する。GUIのアプリ状態として公開フィールドを持ちappが更新する。coreモデルの内部は変更せず、Teamの更新には既存with_*を使う。GuiField/GuiStickは非公開フィールドと読み取り専用アクセサを持つ値とする。
- GamePhaseはReady/Charging/Simulating/Finishedのみ。手番初期化・状態遷移・採点・勝敗判定はモデルへ実装しない。チーム・スキットルのcoreモデルを描画部品へ複製せず、所有元を一か所にする。
- pose.rs、teams.rs、Rapierワールド、サービス、view/controller、GUI起動、追加依存、coreトレイトの変更は今回作らない。目標位置の配列は一時的な素朴な表現であり、物理導入時に採用エンジンの座標型との接続を決める。

### テストリストと進行

- [x] 変更前cargo test --locked --quiet：130件成功、失敗・無視0件。リモートmainと作業元の差分なし。
- [x] Field：通常の12個体・Stand・一意なIdと、渡されたField/個体集合の保持を確認する。
- [x] Stick：未照準/速度0の初期状態、coreの棒のId、確率ではない初速と目標の保持を確認する。
- [x] Game：チームのId・手番を保持し、共通Field/StickとReadyで開始する。
- [x] 未実装APIによるRed→models実装のGreen→全テスト・build・fmt・Clippy・差分を確認する。
- [x] modelsだけをcommit・pushし、次段階へ進まず提示する。

### 実行・レビュー結果

- Red：tests/gui_2d/models/{field,stick,game}.rsの4件を先に追加し、cargo test --locked --test gui_2dで未公開gui_2dモジュールによるE0433を確認した。実行時の振る舞いの失敗とは区別する。
- Green：modelsと公開modを追加し、同じコマンドで4件成功。独自フィールドの同番号個体もIdを作り直さず保持すること、初速6m/sが確率の上限1で制限されないこと、既存のチームId・手番を保持することを確認した。
- Refactor：GameはGUIの状態保持だけとし、coreルールの呼び出しや状態遷移メソッドを追加していない。非公開のField/Stick内部へ可変参照を公開せず、未使用のwith_*や汎用インターフェースも作らない。GuiStickの更新は同じcore::Stickを渡した値の置き換えで表せる。
- 全体：cargo test --locked --quietは134件成功（既存81+49、新規4）、失敗・無視0件。cargo build --locked、cargo fmt --check、git diff --checkも成功した。
- Clippy：cargo clippy --locked --all-targets -- -D warningsは既存のcore/services/molkky.rs:37のneedless_returnで停止する。今回このcoreファイルには差分がない。同じコマンドでこのlintだけをCLI引数-A clippy::needless_returnで除外すると成功した。ソースへのallow追加や既存警告の隠蔽はしていない。
- スコープ：Cargo.toml/lock、core、CUIは変更していない。GUIを起動する入口、入力、view、GuiMolkky、Rapier物理計算は未実装であり、GUIのビルド・実画面操作は検証していない。次段階へは自動で進まない。


## 得点定数をScoreBoardのimplへ移動（2026-11-02）

- 対象：最新main（f03cf1c）のsrc内の定数を全件確認する。TARGET_SCORE/RESET_SCORE/ZERO_POINTS_LIMITをScoreBoardの公開関連定数に移し、内部参照はSelf::へ統一する。モジュールからの再公開は削除する。値・採点・勝敗・入出力は変更しない。
- 他の既存定数はsrc内に存在しないため、追加の集約や新しい定数は不要と判断した。公開APIの利用側はScoreBoard::定数名で参照する。
- 初回の手元には古い履歴が残っていた。通常pushが拒否されたため最新mainを取得し、新ブランチで作り直した。旧履歴のTUI変更や文書は取り込まない。
- [x] 最新mainの既存テストで変更前を確認する。
- [x] 3定数の配置と参照、再公開を更新する。
- [x] 既存テスト・整形・Clippy・差分を確認する。
- 結果：振る舞いを変えない移動なので既存テストを使用し、新規テストやRed確認は追加していない。cargo test --lockedは変更前後とも130件成功（81+49）、失敗・無視0件（/tmp/constants-current-baseline.log、/tmp/constants-current-final.log）。cargo fmt --checkとgit diff --checkも成功した。
- Clippy --locked --all-targets -- -D warningsは変更対象外のsrc/core/services/molkky.rs:37のneedless_returnで停止した。origin/mainにも同じreturn Noneが存在することを確認し、定数移動の差分には含めていない。旧履歴に対する157件の成功結果は最新版の検証結果として使用しない。


## CUIモルックの作成（2026-10-29）

- 対象：`cui/input.rs`で標準入力と再入力、`cui/model.rs`で入力の変換・検証とTeam/Skittle生成、`cui/main.rs`でゲームループ、`cui/output.rs`で表示を担当する。バイナリのmainは標準入出力を渡すだけにする。
- 操作：引数なしではチーム数（1以上）と名前を入力し、入力順に投げる。引数に名前を渡す起動方法も維持する。倒れた番号を空白区切りで入力し、0はミス、qまたはEOFで終了する。フォルト・記録訂正・12投制限・乱数による投てきは今回追加しない。
- 入力方針：名前は制御文字を先に拒否し、trim後に1〜30 Unicodeスカラー値。チームの同名は許可しIDで区別する。投てき番号は1〜12の重複なし、0は単独のみ。不正入力ではゲーム状態を進めず再入力する。
- 得点：既存のScoreBoardの13件の失敗はCUIの得点と失格判定に直結するため、先に修復する。投てき点数の生履歴を保持して順に計算し、3回連続0点は失格・0点、50点超過は25点、50点到達後と失格後の追加は拒否する。入力形式の検証や文字列変換は追加しない。
- 試合：次の手番は既存のMolkkyを利用する。他チームが全て失格した場合は残ったチームを50点の勝者にし、投てきを追加せず終了する。単独参加は50点到達または失格まで続ける。得点表の計算をCUIへ複製しない。
- 制約：既存のviewテストの移動など別途編集中の変更は取り込まない。削除済みのsimulationや旧TUIの復元・テスト無効化は行わない。

### テストリスト

- [x] CUI-1：既存の得点表13件のRedを解消し、投てきの履歴・補正・連続ミス・終了後拒否を保つ。
- [x] CUI-2：チーム数を整数へ変換し、空・0・負数・非整数を拒否する。
- [x] CUI-3：名前のtrim・長さ・制御文字と指定人数を検証し、同名でも別IDのTeam配列を作る。不正な名前では構築途中の状態を変えない。
- [x] CUI-4：投てきの番号を検証してSkittle一覧へ変換する。空・範囲外・重複・0との混在を拒否し、0は空集合にする。
- [x] CUI-5：対話入力の再入力、CRLF、q、途中のEOF、I/Oエラーを扱う。
- [x] CUI-6：表示は注入したWriterへ得点・投数・現在手番・補正・失格・勝者を描画する。
- [x] CUI-7：ゲームループで採点、交代、循環、失格者スキップ、50点到達後の停止を確認する。
- [x] CUI-8：最後の生存者を履歴を増やさず50点の勝者にし、単独失格では勝者なしで終了する。
- [x] CUI-9：コマンド引数の名前検証、実バイナリへの標準入力、q/EOFの終了、無効な入力で投数が増えないことを確認する。
- [x] CUI-10：対象スイート、ビルド、整形・差分、起動例、既存全体テストの残課題を確認する。

- 変更前：サービステスト28件成功、モデル35件成功・13件失敗。モデルの失敗は既存の期待値との差であり、コンパイルエラーではない。修復後に成功した項目だけチェックする。

### 設計・実行・レビュー結果

- 入力モデル：TeamSetupがチーム数・名前・指定人数を検証してTeam一覧を作る。ThrowInputは全番号を検証してからSkittleを生成する。input.rsは行の読み取りと再入力、output.rsはWriterへの表示だけを担当する。ゲームループはBufRead/Writeを受け取り、標準入出力や端末に依存せず検証できる。
- Red：変更前の既存得点表13件の失敗を再実行して確認した。新しいCUIとwin_by_defaultは、テストを先に追加し、未実装APIとバイナリ入口のコンパイル失敗を確認した。新機能の未実装確認と、実際の振る舞いの13件のRedを区別する。テスト内の名前衝突やCargoの入口重複といった準備上のエラーはRed確認には数えない。
- Green：得点表を履歴の逐次評価へ直し、既存モデル48件が成功した。勝利付与の1件を追加してモデル49件成功。CUIはモデル15件、入力9件、表示6件、ゲームループと実バイナリ13件が成功した。新しいケースの初回成功を、実行時のRed確認済みとは扱わない。
- Refactor：変更ファイルだけを整形し、テスト本体を対応するtests/cui/へ置いてmodで階層化した。旧mainテストの名前の拒否条件も移動して維持した。tests/cui/main.rsが別のテスト入口として自動推論されないよう、Cargo.tomlにtests/cui.rsの入口だけを明示した。既存のテストターゲットは無効化していない。
- 検証：`cd rust_molkky && cargo test --locked --test core --test models --test cui --test main`で計120件成功（28・49・30・13件）、無視0件。`cargo clippy --locked --lib --bins --test core --test models --test cui --test main -- -D warnings`も成功した。
- 全体：`cargo test --locked`は、削除済みsimulationと旧rand::RngCoreを参照する既存テストの3件のコンパイルエラーで停止する。既存テストの削除・無効化で成功扱いにはしていない。
- レビュー（高・入力：得点表の既存13ケースとCUIのゲームシナリオ）：補正で過去の投てきを失う、累計のミスで失格になる、失格時に得点が残る、終了後も記録できる問題を修復した。生存者の勝利はScoreBoard::win_by_defaultで表し、実在しない投てきを履歴へ追加しない。
- レビュー（中・入力：入力検証と実プロセスのテスト）：引数の不正な名前は開始前にエラー終了し、対話入力では再入力する。EOF・qは途中の設定や投てきを採用せず終了する。I/Oエラーの伝播、プロンプトのflush、番号重複、名前のUnicode文字数と制御文字を確認した。標準ライブラリだけで新機能を実装し、依存関係は追加していない。
- 最終確認：`cargo build --locked`成功。`cargo run --locked --quiet -- 赤 青`へ`5 8`、`12`、`q`を渡し、2点・12点の記録、交代、終了の表示を確認した。変更したRustファイルへのrustfmtのcheckと`git diff --check`も成功。テストターゲットはcore・models・cui・main・simulationで、従来の4ターゲットにcuiを追加しただけである。AGENTS.md・STEP.md・依存バージョン・Cargo.lockとREADMEのJava部分は変更していない。

## Molkkyのテスト再作成と実装の修復（2026-10-29）

- 対象：Molkkyの実装、対応するサービスのテスト、coreを公開する最小のライブラリ入口、関連するAPI説明。CLI・TUI・描画・シミュレーションは復元せず、得点表の既存13件の失敗や未着手のBONUSも今回修正しない。
- 入力方針：AGENTS.mdの責務変更に従い、ID重複・複数手番・複数勝者などの入力検証をMolkkyへ再導入しない。旧検証テストは新しい契約に合わせて外し、入力側の検証は未実装として残す。13本・同番号別個体の採点をSTEPの正常系にはしない。
- 公開API：hitは注入した判定を1回呼び、所有する結果を返す。calc_scoreはPointを直接返し、0本・1本・複数本の仕様を扱う。judge_gameは50点到達済みのチームを`Option<&Team>`で検索する。最後の生存者の得点を50へ変更する試合進行は含めない。
- next_teamの判断：質問への回答は未受領のため、メソッド名と旧テストの循環規則に合わせ、推奨案の「次のチームを`Option<&Team>`で返す」を仮定として採用した。手番フラグの更新は呼び出し側が行う。現在の次から循環して失格者を飛ばし、未設定なら先頭の有効チーム、単独参加なら同じ有効チームを返す。50点到達・複数参加の最後の生存者・空・全失格ではNoneとする。
- 配置：`src/lib.rs`でcoreだけを公開する。テスト本体は`tests/core/services/molkky.rs`に置き、入口`tests/core.rs`は登録だけとする。

### テストリスト

- [x] M-1／S01-1：1本なら番号1・5・12の点数になる。
- [x] M-2／S01-2：異なる番号の2本・3本は、それぞれ本数の点数になる。
- [x] M-3／S01-3：0本は0点、標準の12本は12点になる。
- [x] M-4：採点後に入力配列を変更しても取得済みのPointは変わらない。
- [x] M-5：hitは棒と全個体を判定関数へ1回渡し、結果の状態・順序を保ち、元の入力を変更しない。空集合も扱える。
- [x] M-6：hitの所有する結果は入力と判定関数の借用期間後も使える。
- [x] M-7：空のtrait実装・dyn Molkkyから既定処理を呼び、呼び出し側で採点・得点表更新ができる。
- [x] M-8：judge_gameは空・未決着ならNone、50点到達済みなら手番に依存せず元のチームの参照を返す。参照の借用期間はチームの一覧に結び付く。
- [x] M-9：next_teamの採用した契約で、選択・該当なし・入力不変を確認する。
- [x] 対象テスト・ライブラリのビルド・モデルの既存失敗との差分・整形・文書を確認する。

- 変更前：`cargo test --locked --test core`はライブラリ入口の欠落による未解決importでコンパイル失敗。仕様のRedとは扱わない。`cargo test --locked --test models`は35件成功・既存13件失敗・無視0件。

### 実行・レビュー結果

- ビルド修復：ライブラリ入口の追加後、`cargo check --locked --lib`で実装のE0308を3件確認した。calc_scoreのPoint/Optionと、judge_game・next_teamの参照/Optionの不一致を直した。入口の欠落とコンパイル修復は、振る舞いのRedとは区別した。
- Red：現在手番の検索を残した状態で`cargo test --locked --test core molkky::next_team`を実行し、11件中2件成功・9件失敗。次のチームを選べない、失格・終了時も現在手番を返す等の違いを確認した。
- Green：循環・失格者スキップ・終了時のNoneを実装し、`cargo test --locked --test core`で28件すべて成功した。採点・hit・勝者検索の17件は型修復後の初回実行から成功しており、Red確認済みとはしない。
- Refactor：hitの一時変数とreturn、勝者検索のfilter/nextを除き、直接の返却とfindへ整理した。整形後もサービステスト28件成功。新しい検証・変換用の型や依存関係は追加していない。
- ライブラリ：`cargo check --locked --lib`成功。ライブラリ入口はcoreのみ公開し、削除済みのアプリを復元していない。
- モデルへの影響：`cargo test --locked --test models`は35件成功・13件失敗・無視0件。変更前後で失敗したテスト名の集合が一致し、新規のモデル失敗はない。
- 全体：`cargo test --locked`は削除済みCLIの`CARGO_BIN_EXE_rust_molkky`欠落、simulationの未解決import、旧rand::RngCore参照でコンパイル失敗。Molkkyの対象成功と全体未通過を分けて扱う。
- レビュー（高・入力：next_teamと新テスト）：現在のチームを返す実装を次のチームの選択へ直し、空でもゼロ除算せず有限回で検索することを確認した。元の手番・チームの参照、サービスより長く借用できる結果も検証した。
- レビュー（中・入力：AGENTS.mdと旧サービスの30テスト）：ID重複・複数手番・複数勝者等のpanic期待、validate_skittles、13本や同番号を正常系とする期待は新方針の対象外とした。入力検証の移植済みやBONUSの試合全体の完了とはしない。S01-1〜3だけを今回の仕様リストで完了にする。
- 差分確認：変更したRustの3ファイルへの`rustfmt --edition 2024 --config skip_children=true --check …`と`git diff --check`は成功。既存ソースはMolkky以外のハッシュが変更前と一致し、AGENTS.md・STEP.md・依存関係・READMEのJava部分も変更していない。テストの無視や登録解除はなく、入口にテスト本体は置いていない。API例全体のドキュメントテストは未登録・未実施とREADMEへ明記した。

## Nameの入力検証・正規化の削除（2026-10-29）

- 承認された範囲：Nameだけを変更する。Skittle・Pointの範囲チェック、得点計算、勝敗判定、別途編集中のファイルは今回変更しない。CLI・TUIの復元や入力側の検証実装は追加しない。
- 責務・API：`Name::new(value: &str) -> Self`とし、保持用のコピーだけを行う。trim、空文字・文字数・制御文字の検証、NameErrorとその再公開を削除する。`as_str`・Display・Clone・等価比較は保持する。
- 契約の変更：Nameは名前の妥当性を保証しない。入力の検証・正規化は入力側の責務であり、今回は未実装として残す。以前の名前の制約を記録した箇所は当時の履歴とし、現在の契約はこの記録とREADME・API例で示す。
- 呼び出し元：現存するテストとAPI例のName生成後のunwrapを除く。名前以外のAPIやテストの期待を変更せず、削除中の実装に対応するCLIの入力拒否テストも残す。

### テストリストと確認手順

- [x] N-1：前後の半角・全角空白、日本語、内部の空白、記号をas_str・Displayでそのまま保持する。
- [x] N-2：空文字と空白だけの文字列をそのまま保持する。
- [x] N-3：31文字の名前も拒否せずそのまま保持する。
- [x] N-4：改行・タブ・その他の制御文字を変更せず保持する。
- [x] N-5：Cloneと同じ文字列の名前の等価比較を保持する。
- [x] Name・Teamの対象テスト、モデル全体の既存失敗との比較、旧API参照、整形・差分を確認する。

- 変更前：`cd rust_molkky && cargo test --locked --test models`は46件中33件成功・13件失敗・無視0件。失敗は既知の得点履歴・失格・終了後の記録のケースであり、今回の修正対象にしない。
- Redは、既存APIでN-1の原文保持を期待して実行し、trimによる相違を確認する。その後、実装と生成APIの呼び出しを更新する。他の名前のケースは同じコンストラクタの契約変更としてまとめ、成功した項目だけチェックする。

### 実行・レビュー結果

- Red：`cd rust_molkky && cargo test --locked --test models name::creation::前後の空白も日本語と内部の空白と記号もそのまま保持する`で1件失敗。期待値`"　 赤 チーム! 🏆  "`に対し実際は`"赤 チーム! 🏆"`となり、trimによる違いを確認した。コンパイル失敗ではない。
- Green：`cargo test --locked --test models name::`は5件成功、`cargo test --locked --test models team::`は6件成功。N-1〜5をチェックした。N-2〜5は新契約へ更新後の初回実行から成功しており、Red確認済みとはしない。
- 整理・Refactor：名前のテストを`storage::whitespace`、`storage::characters`、`value_semantics`で整理し、変更したRustファイルだけを整形した。整形後もName・Teamの対象テストは成功した。Nameの実装は文字列の保持・参照・Displayだけとし、追加の抽象化は導入しなかった。
- モデル全体：`cargo test --locked --test models`は48件中35件成功・13件失敗・無視0件。変更前後の失敗したテスト名の集合が一致し、新規の失敗はない。既存の得点履歴・失格・終了後の記録の未完了チェックは維持した。
- 全体確認：`cargo test --locked`は`rust_molkky`の未解決import、`CARGO_BIN_EXE_rust_molkky`の欠落等、既存の構成エラーで失敗。サービス・CLI・TUI等の全体成功を確認できていない。入力側の検証は未実装であり、この変更で移植済みとは扱わない。
- 差分確認：変更ファイルへの`rustfmt --edition 2024 --config skip_children=true --check …`と`git diff --check`は成功。現存するRustコードとREADME・API例にNameError・旧生成APIのunwrapが残っていないことを確認した。
- レビュー（中・入力：Nameの生成と全呼び出し元）：Nameが空文字・31文字・制御文字を拒否せず保持する新契約と、Resultを要求しない呼び出しが一致した。入力検証の未実装は承認された範囲に従って文書化した。変更前のスナップショットとの比較で、Nameとモデルの再公開以外のプロダクトコード、AGENTS.md、STEP.md、依存関係を変更していないことを確認した。

## modelsのテストリストをテストコードへ反映（2026-10-29）

- 対象：`rust_molkky/tests/core/models/`、テスト登録用の入口、下記テストリスト。プロダクトコード・サービスのテスト・AGENTS.md/STEP.mdの編集中の変更には触れない。
- 責務：モデルのテストではMolkkyの採点処理を呼ばず、点数の準備には実際のPointの生成処理を使う。実ソースを組み込む`tests/models.rs`へモデルの登録を移し、`tests/core.rs`には既存サービスの登録を残す。削除途中のlib/CLIは復元せず、`cargo test --test models`で実作業ツリーのモデルを検証する。
- APIの判断：historyは既存どおり投てき点数の履歴として扱う。過去の得点の期待は、保持された履歴の各接頭部分を公開APIのScoreBoard::newで復元して照合する。テスト側で加算・補正・失格の計算を実装しない。生履歴の圧縮で過去が失われる場合は失敗として残す。
- 拒否方式：既存の終了後拒否テストに合わせてpanicで確認する。STEPがpanicを指定しているわけではなく、Result等を採用する場合は業務上の拒否と入力不変の期待を保って変更する。
- 範囲：既存APIで表現できる基本STEPのケースと、既存のモデルの入力方針・不変更新をmodで整理する。フォルト・連続ミス数の問い合わせ・訂正のAPIは未定義なので、架空の型や無視したテストを追加しない。BONUSの着手は今回含めない。
- チェック条件：下記の各IDに対応するテストがすべて成功し、その項目の振る舞いを既存APIで確認できた場合だけチェックする。モデル全体の成功とは区別する。フォルトを含む項目は通常0点だけの部分確認では完了としない。
- [x] 専用入口へ登録を移し、7つのモデルのテストを意味のあるmodへ整理する。
- [x] STEPと矛盾する履歴・失格得点の期待を修正し、実ソースに対して実行する。
- [x] 成功したIDだけチェックし、失敗理由と未実装APIを記録する。
- [x] 整形・差分・変更範囲を確認する。

### 実行結果とチェックの対応

- `cd rust_molkky && cargo test --test models`：46件を実行し、33件成功・13件失敗・無視0件。コンパイルエラーではなく、実際のモデルの振る舞いによる失敗を確認した。一括復元・補正後の終了判定も既存の退行確認として残した。
- 成功した7項目（対応する11テスト）：S02-1/2/3、S03-1/2、S04-1/3。テスト名の`s02_1_`等の接頭辞でリストと対応させ、該当するテストがすべて成功した項目だけチェックした。
- S01はサービスの採点規則なので今回のモデル検証ではチェックしない。S05〜07とS08のフォルト・連続ミス数の問い合わせ、BONUS A/Bは未実装・未検証として残す。通常0点だけで確認した失格ケースを、フォルトを含むS08-1/4の完了とはしない。
- Name・Id・Field・Stick・Skittleの入力方針や同一性、Teamの得点表の独立性・不変更新は補助的な退行テストとして残した。同番号の別個体を採点するケースと履歴の圧縮を正解にする期待は外し、採点はサービス、補正履歴はscore側の責務として整理した。

| 優先度 | 対応項目・失敗件数 | 入力と実行結果 | STEPの期待・残課題 |
| --- | --- | --- | --- |
| 高 | S04-2：2件 | `[12,12,12,12,3,10]`を逐次追加すると、保存履歴から得られる推移は`[25,35]`。同じ入力をnewで一括復元すると最終得点25 | 推移は`[12,24,36,48,25,35]`、最終得点35。補正前の投てきを保持し、履歴を順に評価する必要がある |
| 高 | S04-4：3件 | 50点到達後の正の点数・0点の追加と、終了後の記録を含む一括生成が拒否されない | 終了後の記録を拒否し、得点・推移を保持する |
| 高 | 補正後の50点到達の復元：1件（補助的な退行確認） | `[12,12,12,12,3,12,12,1]`の逐次追加は50点・Wonだが、一括復元は25点・Reset | 両経路で50点・Wonとなり、それ以前の推移を保持する。終了後の記録も拒否する |
| 高 | S08-1の通常0点部分：3件 | `[12,0,0,0]`で失格後12点、補正後の3連続0点では失格後25点。補正を含む一括復元はDisqualifiedではなくReset | 失格した投てきの終了時点の得点は0。フォルトを含む本項目全体の検証は未着手 |
| 高 | S08-2：1件 | `[0,0,1,0,0]`がDisqualifiedになる | 累計の0点回数ではなく、正の点数で連続を切る |
| 高 | S08-4の通常点数部分：3件 | 失格後に正の点数・0点を追加でき、終了後の記録を含む一括生成も拒否されない | 失格後の記録を拒否する。フォルトの拒否と連続ミス数3の問い合わせはAPI未定義 |

- Red：上記13件の期待値と拒否条件で失敗を確認した。プロダクトコードを直して成功させる作業は今回の依頼範囲に含めず、失敗を残す。最初から成功した33件をRed確認済みとはしない。
- Green：今回のチェック対象11テストは既存実装で成功。モデル全体は未達。
- Refactor：モデルのテストからMolkkyへの依存を外し、登録とテスト本体を分離してmodで分類した。全件成功を保つリファクタリングとする前提は成立していない。
- 全体確認：`cd rust_molkky && cargo test`は、既存のlib/CLI削除に伴う`rust_molkky`の未解決import・`CARGO_BIN_EXE_rust_molkky`の欠落等でコンパイルに失敗。モデル以外の成功確認として扱わない。
- 整形確認：変更したRustファイルへの`rustfmt --edition 2024 --config skip_children=true --check …`は成功。初回検証時のプロダクトコード・AGENTS.md・STEP.md・Cargo.tomlのハッシュは作業開始時と一致した。差分確認中に別途編集中のmolkky.rsの変更を検出したが、今回の変更・コミットには取り込まない。
- 差分確認：`git diff --check`成功。39個のテストリストIDに重複がなく、チェック済み7項目が実行ログの成功ケースと一致することを確認した。登録用ファイルにテスト本体、モデルのテストに無視指定がないことも確認した。

## tests/coreのテストケース再検討（2026-10-29・テストリスト）

### 対象・前提・確認状況

- 初回検討の入力：[AGENTS.md](AGENTS.md)、[STEP.md](STEP.md)、`rust_molkky/tests/core/`の8ファイルと対応するcore実装。当時の対象には静的に69個の`#[test]`宣言があったが、成功を確認できた件数ではない。
- 初回検討ではテスト計画とレビューだけを記録し、既存テスト・ソース・AGENTS.md・STEP.mdは変更しなかった。以後のモデルテストの変更・実行結果は冒頭の記録を参照する。基本STEPの候補と任意のBONUSを分け、未到達の課題を先取り実装しない。
- 期待値はSTEPの振る舞いから決める。AGENTS.mdの「良い例」は簡潔さの方向を示す擬似コードとして読み、その出力・型・条件分岐を仕様や正解値にはしない。読みやすさを優先しつつ、仕様上の検証を削って成功扱いにしない。
- API名、列挙型の種類、`Vec`/`Option`、panicかResultか、UUIDの方式は演習仕様ではない。必要な振る舞いが観測できる最小の契約を決め、その契約に合わせてテストを書く。
- [x] 全8ファイルのケースとSTEPの期待値を照合する。
- [x] 仕様に合うケース・修正候補・不足・ペアで決める事項を分ける。
- [x] mod階層案とテストリストを記録し、数値例・差分を確認する。

### 現状レビューと既存ケースの扱い

| 優先度 | 対象・入力と現在の期待 | STEPを根拠とする判断 | テストの扱い |
| --- | --- | --- | --- |
| 高 | `models/score.rs`の`三回連続0点のチームを失格と判定する`は`[5,0,0,0]`で得点5、`超過は連続零点を中断し補正後の三連続零点で失格になる`は失格後25を期待する | STEP 08では失格時の得点は常に0。`states`の復元ケースにも同じ古い期待がある | 失格判定・終了後拒否は残し、失格時の得点の期待を0へ直す計画にする。フォルトを含むミスも追加する |
| 高 | `models/score.rs`の`投てきの点数を順番どおり履歴に保持する`は`[5,11]`、超過ケースと`models/team.rs`は履歴`[25]`を期待する | STEP 03・04で確認するのは各投てき終了時の得点。`[5,11]`なら推移は`[5,16]`、超過前の推移も確認可能である必要がある | 生の点数履歴を持つこと自体は否定しない。ただし、それだけを得点推移の検証にせず、履歴を捨てることを要求しない。推移・投てき結果を別々に観測する |
| 高 | 基本ケースにフォルトを表す入力・結果の区別、36/37点境界、連続ミス数の問い合わせがない | STEP 05〜07の主要な受入条件が不足する | 数値0とフォルトを区別するケース、37点ルール、連続ミス数を追加候補にする。Pointだけでフォルトを代用しない |
| 高（BONUS着手時） | `services/molkky.rs::team_selection`の`勝利状態でなければ最後の生存者や全失格でも勝者を返さない`は最後の生存者もNoneとする | BONUS B-1/B-2では他チームの失格で残ったチームが50点になり勝つ | Wonだけを探す内部問い合わせのテストと、試合全体の判定を分ける。試合の受入条件としては「生存者50点・勝利・終了」を検証する |
| 中 | `services/molkky.rs`の同番号2本・13本を採点できるテスト、`models/skittle.rs`の同番号別個体を採点する期待 | STEPの用具は1〜12各1本。これらは仕様にない変則構成の成功例である | 正常系から外す候補。2本・3本・12本は異なる番号で確認する。不正入力として拒否するか等はペアで決め、無条件にpanicや受理を仕様にしない |
| 中 | `services/molkky.rs`のhitの注入・呼出回数・ID集合比較・借用期間、Field/Stick/UUID v4の検証 | 得点記録の仕様は倒れたスキットルの結果を入力とし、位置・物理判定・UUIDの形式を要求しない | 必須のSTEP受入テストと分離する。現在の公開APIを残す間の補助的な退行テストにはできるが、API自体を削る判断の後まで固定しない |
| 中 | 名前1〜30文字、trim、制御文字拒否、ID重複拒否、複数手番/勝者拒否、空の参加者、単独参加時の循環 | STEPが明示していない入力方針・設計上の取り決めを含む。BONUS B-1の対象は2チームから | 合意済みの独自ルールは少数の境界例で残し、未合意のものは決定待ちにする。単独参加を試合仕様の必須ケースにしない |
| 低 | 得点の不変更新を`score.rs`と`team.rs`等で何度も検証し、一つのテストで無関係な状態を多数確認する箇所がある | 仕様の読みやすさと失敗箇所の分かりやすさを優先する | 採点規則は採点側、得点推移・終了は得点記録側、チーム独立性は試合側へ寄せる。不変更新を選ぶ場合は各公開境界の代表例を残す |

`models/score.rs`と`models/team.rs`は最終得点や履歴保持の検証も含むため、まとめて削除しない。仕様に合う0点開始・加算・49/50/51点・補正後の継続・終了後拒否は再利用する。一方、型のコピー方式や`history()`の内部表現といった期待は、公開する仕様の確認に必要な範囲へ絞る。

### modの階層案と配置

テスト本体は引き続き対応するファイルへ置き、入口は登録だけとする。「対象の振る舞い→前提条件」の順で必要な深さまでmodを分ける。以下は初回検討の案であり、今回のモデル用入口への登録移動と実行範囲は冒頭に記録した。

```text
tests/core/services/molkky.rs
  scoring
    no_skittles / one_skittle / multiple_skittles
  input_policy                    # 合意した入力方針だけ
  match_rules                     # BONUS Bに着手した場合
    two_teams / multiple_teams / throw_limit

tests/core/models/score.rs
  recording                       # 空・加算・0点
  score_history                   # 各投てき終了時の得点
  target_score                    # 49・50・超過・終了後
  faults                          # 0点との区別・結果の記録
  fault_reset                     # 36/37境界・通常0点との違い
  consecutive_misses              # 初期値・連続・中断
  disqualification
    third_miss / priority / recording_after_end
  correction                      # BONUS Aに着手した場合

tests/core/models/team.rs
  creation / independent_scores   # 採点規則を重複して検証しない

tests/core/models/skittle.rs
  valid_numbers / invalid_numbers # 拒否方式は入力方針に従う
```

`name.rs`・`id.rs`・`field.rs`・`stick.rs`はSTEP由来の必須ケースではない。必要性と残すAPIを先に決める。残す場合は生成・同一性・入力方針などのmodで整理し、UUID生成器等の依存ライブラリの振る舞いを細かく再検証しない。投てき結果の新しい型を採用する場合、そのテストは採用したソースファイルと同名のファイルへ置く。

### 基本STEPのテストリスト

チェック済みは冒頭の実行で項目全体を確認できたもの、未チェックは失敗中または未実施の候補を表す。既存ソースに似たケースがあるだけでは確認済みとしない。各STEPへ到達した時点で原則一つずつ選ぶ。数字の列は各投てきの点数、`F`はフォルト、矢印の右側は各投てき終了時の得点を表す。数字25は補正後の得点であり、1回の投てきの入力にはしない。

#### STEP 01：投てきの点数

- [x] S01-1：1本の番号1・5・12を、それぞれ1・5・12点として採点する（既存ケースを再利用）。
- [x] S01-2：異なる番号5・8の2本は合計13ではなく2点。異なる3本は3点。
- [x] S01-3：0本は0点、1〜12各1本の12本は12点。正常系の上下限を確認する。

#### STEP 02：現在の得点

- [x] S02-1：投てきがなければ0点。
- [x] S02-2：5本を倒す投てきの後に11番を1本倒すと16点。同じ点数を別の投てきで2回記録しても両方を加算する。
- [x] S02-3：途中の0点は得点を変えず、1回の投てきとして数える。同じ番号を別の投てきで再び倒せる（立て直しを数値入力の記録で確認し、物理シミュレーションは要求しない）。

#### STEP 03：得点の推移

- [x] S03-1：`[5,11,0,3] → [5,16,16,19]`。途中の0点でも履歴の1行を残す。
- [x] S03-2：記録なしの推移は空。過去の時点を問い合わせても現在の得点や他の記録を変更しない。

#### STEP 04：50点と25点への補正

- [x] S04-1：49点では継続可能。48点から2点で50点となり、到達を判定できる。
- [x] S04-2：`[12,12,12,12,3,10] → [12,24,36,48,25,35]`。補正前後の推移をすべて確認できる。
- [x] S04-3：49点から2点で25点へ戻り、その後も記録できる。25点から再び超過した場合も、その投てきの終了時点で25点になる。
- [ ] S04-4：50点到達後は正の点数・0点とも追加できず、得点・推移を変更しない（STEP 05到達後はフォルトも同様）。拒否の表現方法は合意したAPIに従う。

#### STEP 05：フォルトと投てき結果

- [ ] S05-1：22点でフォルトすると得点は22点のまま。結果はフォルト、点数は0点で、投数・推移は1件増える。
- [ ] S05-2：通常の0点とフォルトを、結果の問い合わせで区別できる。いずれもその後の記録を続けられる。
- [ ] S05-3：例の`[12,10,F,4,12,11,3,0] → [12,22,22,26,38,49,25,25]`と、投てき結果の順序を確認する。フォルトのために倒れた個体の情報を追加で要求しない。

#### STEP 06：37点ルール

- [ ] S06-1：フォルト直前が36点なら36点のまま、37点なら25点、38点以上でも25点へ戻る。通常の点数加算とは区別する。
- [ ] S06-2：37点で通常の0点を記録しても37点のままであり、フォルト時だけ補正する。
- [ ] S06-3：`[12,12,F,12,4,F] → [12,24,24,36,40,25]`。補正時も結果はフォルト（0点）のままで、元の推移を失わない。

#### STEP 07：連続投てきミス数

- [ ] S07-1：記録なしは0回。通常の0点・フォルトはいずれも1回のミスとして数える。
- [ ] S07-2：`[0,F,7,0]`の連続ミス数は各回`[1,2,0,1]`。正の点数で連続が切れる。
- [ ] S07-3：37点ルールで25点へ補正されても、その投てきはミス1回であり、連続ミス数をリセットしない。

#### STEP 08：3回連続ミスで失格

- [ ] S08-1：`[12,0,F,0] → [12,12,12,0]`。2回までは継続でき、3回目で失格・得点0となる。過去の推移は保持する。
- [x] S08-2：途中に正の点数があれば連続は切れ、累計3回のミスだけでは失格にしない。例えば`[0,0,1,0,0]`は未失格。
- [ ] S08-3：`[12,12,12,2,0,0,F] → [12,24,36,38,38,38,0]`。37点以上でのフォルトでも、3回目のミスなら25点補正より失格の0点を優先する。
- [ ] S08-4：失格後は正の点数・0点・フォルトを追加できず、記録・得点・推移を変更しない。失格後の連続ミス数は3である。

### 境界と優先順位の確認表

セット継続中を前提とする。終了後の入力は、下表の計算より先に拒否する。

| 投てき直前の得点 | 直前の連続ミス | 今回の結果 | 今回終了時の得点 | 連続ミス | 終了・失格 |
| --- | --- | --- | --- | --- | --- |
| 36 | 0 | F | 36 | 1 | 継続 |
| 37 | 0 | F | 25 | 1 | 継続 |
| 37 | 0 | 通常0点 | 37 | 1 | 継続 |
| 38 | 2 | F | 0 | 3 | 失格 |
| 49 | 0 | 1点 | 50 | 0 | 50点到達で終了 |
| 49 | 0 | 2点 | 25 | 0 | 継続 |

### BONUS：着手を選んだ場合の候補

BONUS AとBは独立した任意課題とし、基本STEPの完了条件へ混ぜない。

#### BONUS A：記録の訂正

- [ ] A-1：n回目の点数とフォルトを相互に訂正でき、前後の順序と投数は保たれる。訂正以降の得点・推移・結果・連続ミス数をすべて再計算する。
- [ ] A-2：`[12,12,12,1,F]`の4回目を0点へ訂正すると、推移は`[12,24,36,36,36]`、連続ミス数は2となる。訂正前の37点フォルト補正を引き継がない。
- [ ] A-3：50点到達後でも既存記録を訂正できる。`[12,12,12,12,2]`の最後を1点へ訂正すると49点となり、50点到達による終了状態が解除される。
- [ ] A-4：失格後でも既存記録を訂正できる。`[12,0,F,0]`の2回目を1点へ訂正すると推移は`[12,13,13,13]`、連続ミス数は2となり、失格が解除される。
- [ ] A-5：訂正によって終了時点より後の記録が生じる場合は、下記の決定事項を確定してその扱いを検証する。訂正の拒否・保持・切り詰め等を仕様の必須事項として先に決めない。

#### BONUS B-1：2チーム対戦

- [ ] B1-1：指定した先攻から交互に投げ、得点・連続ミス数はチームごとに独立する。他チームの正の点数で自分のミス数はリセットされない。
- [ ] B1-2：Aが48点・Bが30点のとき、Aの2点でAが50点となり勝利・終了する。以後はBも含めて記録できない。
- [ ] B1-3：相手が失格すると残ったチームの得点は50点になり、勝利・終了する。失格者は0点。勝者の元の得点を残したままにしない。

#### BONUS B-2：3チーム以上

- [ ] B2-1：指定した順序で巡回する。失格者の飛ばし方を合意し、その順序とチームごとの記録の独立性を検証する。
- [ ] B2-2：他の全チームが失格すると最後の生存者が50点となり勝利・終了する。まだ複数の生存者がいれば、この理由では終了しない。

#### BONUS B-3：各チーム12投まで

- [ ] B3-1：各チームの投数を管理し、全チームが12投を終えるまでは投数だけを理由に終了しない。12投済みチームに13投目を記録しない。
- [ ] B3-2：投数上限より先に50点到達・失格による決着が起きた場合は、その終了条件を適用する。
- [ ] B3-3：全チームが12投で未決着なら終了し、得点の降順に順位を求める。Aが毎回2点・Bが毎回1点・Cが毎回3点の12投なら、36点のC、24点のA、12点のBの順となる。
- [ ] B3-4：通常0点・フォルトも1投に数える。同点順位と失格者の投数の扱いは、合意した方針を個別に検証する。

### ペアで決める事項と補助テスト

| 決定事項 | 現在の既存テストとの関係 | 決定後の確認候補 |
| --- | --- | --- |
| 番号範囲外・同じ番号/個体の重複・13本以上など、正規の12本から作れない採点入力 | 現状は番号の範囲外をpanic、同IDを拒否しつつ同番号別個体と13本を受理する期待がある | 受理/拒否と拒否の表現を決めて代表例だけ残す。正常な0〜12本の点数をNone等で欠落させない |
| 不正な記録、終了後の記録、無効な訂正位置の拒否方式 | 既存はpanicとメッセージ文字列に依存するものがある | 方針が合意済みなら維持する。業務上の拒否と入力不変を中心に検証し、STEPの必須事項として特定の文字列を固定しない |
| BONUS Aで訂正後の終了時点より後に記録が残る場合 | STEPが明示的にペアの相談事項としている | 訂正の拒否、記録の保持、切り詰め等から方針を決め、その取り決めどおりに記録を扱うことを確認する |
| BONUS Bの先攻・失格者スキップ・名前/ID・不正な参加者一覧 | 空/単独参加、手番未設定時の先頭選択、同名、UUID、複数手番等が独自ルールとして存在する | 試合を成立させる最小の取り決めを残す。手番フラグの配列更新やVecの返却自体ではなく、次に記録できるチームを確認する |
| BONUS B-3の失格者の12投・同点順位 | STEPが明示的にペアの相談事項としている | 失格者を投数完了とみなすか、順位を同順位にするか等を決める。UUIDや登録順による勝手なタイブレークを追加しない |

`hit`等の任意APIを残すなら、判定を1回呼ぶ・入力を変更しない・所有する結果を保持できる等の最小の契約を`services/molkky.rs::integration_contract`へ分ける。ID集合の比較やエラー処理の網羅を基本STEPのテストリストへ混ぜず、合意した公開契約の範囲で退行を確認する。擬似コードのAPIに合わせるだけの目的で、正しい期待値や拒否条件を弱めない。

### 初回検討時の着手順と検証結果（当時の記録）

1. 実行できる最小のライブラリ入口とcoreテストの登録方法を確認する。削除途中のCLI・描画・シミュレーションを、この検討のために復元しない。
2. 取り組むSTEPを確定し、S01から到達済みの範囲の小さなケースを一つずつ選ぶ。既存ケースの修正では、仕様変更による期待値の修正と、実装の退行を区別する。
3. S03の推移、S05のフォルト、S06の37点境界、S07の連続ミス、S08の失格0点を順に検証する。採用するAPIや新しい型はこの過程で必要な分だけ決める。
4. 実装時はRedを確認してからGreenへ進み、成功を保ってmodの整理や重複の統合を行う。BONUSは選択後に別のリストとして着手する。

- 実行済み：`cd rust_molkky && cargo test`。`src/lib.rs`・`src/main.rs`が削除途中のため、coreでは`rust_molkky`の未解決import、CLIでは`CARGO_BIN_EXE_rust_molkky`の欠落等でコンパイルに失敗した。ログは`/tmp/core-test-reconsideration-baseline.log`。これは環境/構成の失敗であり、仕様のRedや69件の失敗/成功確認として数えない。
- 静的確認：現在の`calc_score`はOption<Point>、旧テストはPointを前提とし、`next_team`もOptionとVecの契約が食い違っている。テストを現在の戻り値へ機械的に合わせず、STEPの観測可能な振る舞いと決めたAPIを先に合わせる必要がある。
- 未実施：テスト・プロダクトコードの追加/変更、各候補ケースのRed/Green/Refactor。今回の完了条件はテスト計画の文書化であり、STEPの機能実装やテスト通過ではない。

- 文書確認：候補39項目のIDが重複せず、全STEPと任意BONUSを分けていること、8つの数値例の計算、git diff --checkを確認した。計画の数値例の照合であり、Rustのテスト実行・Green確認ではない。既存テストとAGENTS.md/STEP.mdは作業前の内容を維持した。確認中に別途編集中のmolkky.rsの変更を検出したが、今回の操作はこの文書だけに限定した。

## テストのmod階層化と講演メモとの整合確認（2026-10-29）

- 対象・入力：AGENTS.mdとSTUDY_AI.md。ユーザー指定のmod階層化を運用規則へ追加し、講演の要点から逸れていないか確認する。Rustのソース・既存テスト・講演メモは変更しない。
- [x] 両文書の要点と追加の運用規則を照合する。
- [x] テストのmod階層化とJUnitのNestedとの対応をAGENTS.mdへ明記する。
- [x] 文書差分・参照先・git diff --checkを確認する。

### レビュー結果

| 根拠となるSTUDY_AI.mdの節 | AGENTS.mdの対応 | 結果 |
| --- | --- | --- |
| 5・6：仕様とアーキテクチャ、テストスイートの重視 | 振る舞い・制約・完了条件を先に整理し、仕様を表すテストを保つ | 整合 |
| 7・8：テストリスト、Red/Green/Refactor、作業履歴 | 原則ひとつ選択、講演の複数選択は理由を記録し、各段階を検証・記録する | 整合。講演の提案と通常の進め方を区別している |
| 3・4：目標以外の制約、レビューの限界 | 小さな差分、範囲の明示、自律的な調査・変更・検証 | 整合 |
| 9・10：検証の選択肢、影響度に基づくリスク | 検証手法・依存関係は目的に必要なものだけ追加し、リスクに応じて確認する | 整合。講演で挙がった手法を一律必須にはしていない |
| 11〜13：技術負債・認知負債・意図負債 | 判断理由・制約・不明点を文書やテストに残し、説明できる状態を保つ | 整合 |

- 指摘・中：modの階層化、Rustの配置・コマンド、自動コミット・push等は講演メモに直接書かれていない。講演の必須事項と混同されないよう、リポジトリ独自の運用規則であることを冒頭へ明記する。
- 判断：modのグループ化は、JUnitのNestedと同様に検証対象・前提条件を整理する目的で使う。JUnitの実行方式・ライフサイクルまで同じとする主張は加えない。既存のtests配下への配置・登録用ファイルにテスト本体を置かない規則と併用する。
- 結論：講演の主旨との矛盾は見つからなかった。独自ルールの出所を明確にする修正であり、STUDY_AI.mdへの追記や既存テストの再構成は今回の対象外。
- 検証：文書のみの変更のためRustテストとRed/Green/Refactorの実行は対象外。別途編集中のSTEP.md、rust_molkky/DEVELOPMENT.mdとソースの変更・削除は取り込まない。

- 確認結果：git diff --check成功。STUDY_AI.mdはHEADとバイト単位で一致し、変更していないことを確認した。

## judge_gameのコンパイルエラー修正（2026-10-29）

- 対象：`rust_molkky/src/core/services/molkky.rs`と呼び出し側。戻り値は`Option<&Team>`を維持する。
- 指摘・高：未定義の`team`の参照、対応しない閉じ括弧、勝者なしの場合の戻り値不足でコンパイルできなかった。
- 判断：50点到達を優先し、2チーム以上の参加者のうち生存者が1チームだけならその参照を返す。得点の更新や新しい結果型は追加しない。CUI側の重複した生存者判定を除去する。
- [x] 50点到達済みのチームを返す。
- [x] 2チーム・3チームで最後の生存者を返す。
- [x] 空・単独参加・全失格・生存者が複数の場合は勝者なしになる。
- 検証：`cd rust_molkky && cargo check --locked --lib --bins`成功。`cargo test --locked --test core`は31件成功。
- Red/Green：構文・型エラーの解消を確認した。コンパイルエラーを振る舞いのRed確認としては扱わない。生存者判定の追加テストは成功を確認した。
- 全体検証：`cargo test --locked`は既存のsimulationテストのimport不整合と、modelsテストが呼ぶ`win_by_default`の欠落によりコンパイル失敗。別途編集中のScoreBoardとviewテスト配置は変更しない。
- 制限：不戦勝時の50点付与は今回の修正に含めない。

## coreのコメント削除（2026-10-29）

- 対象：`rust_molkky/src/core`以下の通常コメント・ドキュメントコメント。ユーザー編集中の処理とテストは変更しない。
- [x] コメントを全て削除し、コメントを除く各行が変更前と一致することを確認した。
- [x] 変更前後の既存テスト結果を比較し、同じ22件成功・9件失敗であることを確認した。
- 変更前：`cargo test --manifest-path rust_molkky/Cargo.toml --locked --test core`は22件成功・9件失敗。既存の失敗として記録する。
- 検証：`cargo check --manifest-path rust_molkky/Cargo.toml --locked --lib --bins`成功。コメント検索は該当なし。処理変更のないリファクタリングのためRed/Greenは対象外。既存の9件の失敗は未修正。

## Molkkyのhit抽象化とrun追加の設計レビュー（2026-10-29）

- 対象・入力：`src/core/services/molkky.rs`、`src/cui/main.rs`と`hit`の呼び出し元。今回は設計の検討のみで、ソースとテストは変更しない。
- [x] `hit`の実装と呼び出し元を確認する。
- [x] ゲームループの責務と`run`の配置を検討する。
- 指摘・中：現在の`hit`は受け取った`JudgeStrategy`をそのまま実行するだけ。判定方法を実装型に固定するなら、判定を必須メソッド`hit`に実装し、Strategy引数と型を削除する方が呼び出し側は単純になる。判定を呼び出しごとに交換する要件がある場合には、現行方式にも用途がある。
- 設計案（後のユーザー説明で戻り値の意味を訂正）：`hit`は棒とスキットルを受け取り、倒れた個体の一覧を返す。乱数源や設定は必要とする実装型が保持する。内部状態を更新する実装が必要なら`&mut self`を選ぶ。必須メソッドへ変える場合、`SimpleMolkky`とテスト内の空の実装にも具体的な判定が必要になる。
- 指摘・中：現在のCUIは`hit`を呼ばず、入力された投てき結果を採点している。CUIのためだけに未使用の判定実装を増やす必要はない。採点用の型と投てき判定の関係は、実装変更時に確認する。
- 判断：現在の`cui/main.rs::run`は入出力・終了操作・ゲームループを扱う。これをドメインサービス`Molkky`の必須メソッドにはしない。現状は既存の関数を維持し、複数の実行環境を同じ実行契約で呼ぶ必要が生じた場合に、外側の実行用traitを検討する。
- 検証：静的な設計レビューのみ。ソース変更がないためRustテストとRed/Green/Refactorは実施していない。上記は提案であり、実装済みではない。

## CUIのmodels・services・run分割案のレビュー（2026-10-29）

- 対象・入力：ユーザー提案のTeams・Field・CuiMolkky・Input・Output・runと、現在の`src/cui`、`src/core/services/molkky.rs`、CUIテスト、README。単一プロセスの対話型得点記録を前提とする。
- [x] データの所有者、入力から採点・描画への流れ、既存の呼び出し元を照合する。
- 提案する責務：`models/teams.rs`のTeamsは完成したチーム一覧、`models/field.rs`のFieldは全12本を所有する。`services/input.rs`のInputは読み取り・変換・入力検証・再入力、`services/output.rs`のOutputは描画とflushを扱う。`services/molkky.rs`のCuiMolkkyはMolkkyの投てき判定を実装する。`run.rs`はこれらを呼び、得点・手番の更新と終了を進める。
- 指摘1・高（レビュー時点の既存不具合。下記追記で複数生存者の判定修正を確認）：`core/services/molkky.rs:20`のjudge_gameは、生存者が複数でも先頭を勝者として返す。2チームが共に未失格でもnext_teamがNoneとなり、現行CUIは最初の投てきで終了する。新しいCuiMolkkyでもデフォルト実装を使えば引き継ぐ。2チーム以上の参加者のうち生存者が1チームだけか確認する必要がある。ディレクトリ移動による不具合ではない。
- 指摘2・中（設計上の未確定事項。Field自体の状態更新は必須ではなく、下記追記で整理）：現在のThrowInputは倒れた番号だけからSkittleを新規生成する。これをそのままFieldに代入すると、例えば5番の入力で全12本が1本になり、個体IDも引き継がない。Fieldを全12本の所有者と定義し、入力番号で既存の個体を選ぶ。採点には全12本ではなく今回倒れた本だけを渡し、次の投てきは立った状態から扱う。
- 指摘3・中（設計上の未確定事項。入力をhitの外に限定する提案は下記追記で訂正）：現在のhitはVec<Skittle>だけを返す一方、入力はq・EOFによる中断とio::Errorを返す。Inputをhit内へ隠すと中断を表せず、空Vecで代用すればミスとして記録してしまう。現行のhitの戻り値を維持する案では、runが入力の中断・失敗を処理し、CuiMolkkyへ検証済みの投てき情報を渡す。
- 単純化：InputとOutputを構造体にするなら、それぞれ読み取り元・書き込み先を保持する。TeamsとFieldの同じ一覧をrunやCuiMolkkyにも複製して所有させない。ゲームループはrunだけに置き、CuiMolkkyへ重複させない。チーム設定中の予定数はInput側のローカル状態で管理でき、完成したTeamsのフィールドを増やす必要はない。
- 判断：提案のディレクトリ構成を採用してよい。指摘1は動作上の修正が必要で、指摘2・3は実装前に責務を確定する。新しいtraitやStrategyの再導入は提案しない。
- [ ] 実装時：2チームが交互に投てきし、勝利条件を満たすまで続くことを確認する。
- [ ] 実装時：5番の入力後も全12本と個体IDを保持し、採点は5点になることを確認する。
- [ ] 実装時：次の投てきで前回の転倒を持ち越さず、q・EOF・入力エラーでは得点と手番を進めないことを確認する。
- 検証：静的レビューのみ。ソース・テストは変更しておらず、RustテストとRed/Green/Refactorは実施していない。配置・型・責務は提案であり未実装。

### ユーザー説明を受けた補足

- [x] `judge_game`が生存者をfilterした後に`exactly_one()`を使う変更を静的に確認した。生存者が複数いる場合は返さなくなった。参加自体が1チームの場合も勝者を返すので、単独練習を継続するなら参加数の条件は別途必要。
- `hit`の契約は「全スキットルを受け取り、今回倒れたスキットルだけを返す」。前回の説明には更新後の全体を返す前提が混ざっていた。Fieldは全12本を保持し、hitの戻り値をその一覧へ代入しない。5番だけ倒したら戻り値は1本で5点、次に8番と12番なら2本で2点。どちらもFieldの一覧は12本のまま。
- この契約なら、Fieldの各個体の転倒状態を更新することは必須ではない。既存の個体から倒れた本を選んで返せばIDも維持でき、次の投てきにも同じ全12本を渡せる。倒れた本だけという意味を持つ戻り値と、Fieldが保持する全体を区別するための説明であり、ユーザー案に全体を置き換える処理があると認定したものではない。
- 入力やエラー処理をCuiMolkkyのhit実装内で完結させてもよい。Inputが形式検証と再入力、Outputが案内を担い、それらをhitから利用できる。入力を必ずrunで先に済ませる必要があるという前回の制限を訂正する。
- `Vec<Skittle>`で十分なのは、実装側で処理を完結し、正常な投てき結果だけを返す場合。q・EOFでrunへ終了を通知する要件まで含める場合は、その通知方法を決める必要がある。空Vecは0本の投てきとして使うため、終了の代用にしない。新しい戻り値型は今回決めず、ソースも変更しない。
- 検証：ソース参照による確認のみ。テスト未実施。

## Molkkyへのaim抽象メソッド追加（2026-10-29）

- 対象：`src/core/services/molkky.rs`。最新のユーザー指定に従い、デフォルト実装を持たない必須メソッドを追加する。
- 公開API：`aim(&self, stick: &Stick, skittles: &[Skittle]) -> Vec<Skittle>`。戻り値は狙う個体の一覧とし、`hit(stick, &targets)`へ渡す。選択方法・入力・検証は実装側が担当する。
- [x] aimの抽象宣言だけを追加し、編集中の既存処理を維持した。
- [x] 実ソースのcoreモジュールを一時的なrustc入力から参照し、`&dyn Molkky`でaimの戻り値をhitへ渡す呼び出しも含めて型検査に成功した。
- 変更前の全体確認：`cargo check --manifest-path rust_molkky/Cargo.toml --locked --lib`は既存CUIの`SimpleMolkky`未定義で失敗した。今回の対象外として処理を変更しない。
- 変更後の検証：`cargo check --manifest-path rust_molkky/Cargo.toml --locked --lib`と`cargo test --manifest-path rust_molkky/Cargo.toml --locked`は既存のSimpleMolkky未定義により失敗。抽象宣言だけで実装の振る舞いを追加していないため、新規の実行テストとRed/Greenは対象外。選択処理とCUIへの接続は未実装。

## CUIのmodels・services・io・runへの再構成（2026-10-29）

- 対象：`rust_molkky/src/cui`、起動元、対応テストとREADME。編集中のcoreの処理を変更せず、現在のaim/hit抽象メソッドを実装する。
- 設計：InputとOutputはDDDのプレゼンテーション層に当たり、`cui/io/`へ配置する。Inputは読み取り元、Outputは書き込み先を持つ。TeamsはVec<Team>、FieldはVec<Skittle>を持ち、Fieldは1〜12番を一度生成して保持する。CuiMolkkyは検証済みの番号を保持し、aimで既存個体を選び、hitで選んだ本を転倒結果として返す。入力・中断・ioエラーはioモジュール、ゲーム進行はrunが扱う。
- このCUIは倒れた番号を記録する方式を維持する。入力された番号をaimで個体に対応付け、hitはその個体を倒れた本として返す。命中率や乱数による投てきは追加しない。
- [x] Teams・Field・CuiMolkky・Input・Outputを指定の責務で配置し、runと起動元を接続する。
- [x] aimからhitへ渡す選択結果・個体ID・入力順・空の選択・元のField不変を確認する。
- [x] チーム数・名前・投てき番号の既存の入力検証を維持し、対応するファイルへテストを移す。
- [x] q・EOF・ioエラー・プロンプトのflushを維持する。
- [x] CUIと既存ゲームループのテスト、ビルド、差分を確認して結果を記録する。
- 変更前の状態：既存CUIが削除済みSimpleMolkkyを参照してコンパイルできない。coreのScoreBoard・Molkky・services/mod.rsはユーザー編集中であり、採点と勝者・手番の判定処理自体は今回変更しない。

### 実装・検証結果

- Red：`cargo test --locked --test cui molkky::`で、選択せず全12本を返す最小実装に対して4件とも期待した理由で失敗した。選択・空の選択・個体ID保持・投てき間の独立性は同じaimの選択処理に依存するため、まとめて選択した。変更前のSimpleMolkky未定義によるコンパイル失敗はRed確認には含めない。
- Green：入力番号に対応する既存個体を入力順に選ぶ実装にし、同じ4件が成功した。hitの結果だけにFallを設定し、Field全体は保持する。
- Refactor：テスト本体も`tests/cui/models/`・`services/`・`io/`・`run.rs`へ移動した。旧TeamSetupの予定人数はInput内で管理し、旧ThrowInputの検証はInputへ移した。名前の入力規則はCUIモデルTeamsが保持する。旧main/input/output/modelモジュールの互換ラッパーは追加しない。
- `cargo check --manifest-path rust_molkky/Cargo.toml --locked --lib --bins`成功。実バイナリの標準入力操作、名前の引数、設定途中EOFの既存テストも成功した。
- `cargo test --manifest-path rust_molkky/Cargo.toml --locked --test cui`：29件成功・1件失敗。`--test main`：9件成功・4件失敗。失敗した期待値は削除・緩和していない。

| 未解決の検証 | 入力・期待値と原因 | 優先度 |
| --- | --- | --- |
| 出力：補正後も5投を表示 | 12,12,12,12,3の履歴がScoreBoardで25点1件に置き換わるため、5投の表示にならない | 高・既存core |
| run：単独参加の継続・連続ミス・超過後の継続（3件） | judge_gameが未失格の単独参加者も勝者として返すため、最初の投てきで終了する。ScoreBoardの累計ミス・履歴消失の問題も未解決 | 高・既存core |
| run：最後の生存者に50点 | judge_gameは勝者を返すが、残ったチームへの50点付与は未実装 | 高・既存仕様差分 |

- 全体の`cargo test --locked`は、旧SimpleMolkky・JudgeStrategy・空のMolkky実装・削除済みsimulation・rand::RngCore・win_by_defaultを参照する既存テストでコンパイル失敗した。今回のCUI再配置とは別の残課題として記録する。
- ソース比較：作業開始時のスナップショットとcore以下の全Rustファイルが一致することを確認した。CuiMolkkyが依存する既存のaim/hit抽象化とservicesの公開API変更はコミットへ含め、別途編集中のScoreBoard変更は取り込まない。
- 最終確認：`cargo build --manifest-path rust_molkky/Cargo.toml --locked`成功。対象ファイルのrustfmtチェックと`git diff --check`成功。コミットにはCuiMolkkyに必要なhit抽象化・旧Strategy/実装型の公開終了だけを含め、ユーザー編集中のjudge_gameとScoreBoardの変更は除外する。

## Rust得点表・Javaテストのエラー修正（2026-10-29）

- 対象：`rust_molkky/tests/core/models/score.rs`と対応する得点表実装、勝敗判定との責務整理、`java_molkky/src/test/java/molkky`の実行確認。
- 設計：得点表は自チームの投てき履歴・得点・終了状態を扱う。他チームの状態から勝者を決める責務はMolkkyに保つ。旧win_by_defaultのテストは、判定結果を反映するwith_winへ変更し、50点・履歴維持・終了後の記録拒否という期待値はすべて維持する。サービス側には、勝者判定だけでは元の得点表を変更しない検証を追加する。
- [x] 得点表テストの旧API参照を整理し、失敗を実行できる状態にする。
- [x] 補正前後の履歴、連続ミス、失格の0点、終了後記録拒否を確認する。
- [x] JDK 17.0.21+9をJAVA_HOMEで指定してJavaの全28件が成功。失敗・エラー・スキップ0件をJUnit XMLから確認した。
- [x] 関連テスト、差分、未解決事項を確認し記録する。
- 変更前：Rustのmodelsテストはwin_by_default欠落でコンパイル失敗。JavaのGradle起動はJAVA_HOME未設定・java未検出で失敗。どちらも振る舞いのRedとしては数えない。
- Javaの詳細は`java_molkky/DEVELOPMENT.md`へ記録した。Javaソースとテストは修正不要だった。

### Red・Green・Refactorと結果

- Red：旧API参照を整理した後の得点表テストは14件成功・13件失敗。履歴の逐次再生に必要な補正、連続ミスのリセット、失格の0点、終了後記録拒否が期待した理由で失敗した。同じ履歴再生処理の修正で確認するため、既存の関連ケースをまとめて選択した。
- Green：newで不変のfoldを使って履歴を順に計算し、元の履歴と現在の得点を分けて保持する。addは新しい得点表を返し、元の状態を変更しない。
- 勝利反映のRed：with_winが元の得点表を返すだけの最小実装では、13点から50点になる期待値で失敗した。履歴を維持して50点・Wonにした新しい得点表を返す実装で成功した。勝者を選ぶ判断はjudge_gameに残した。
- ゲームループのRed：得点表修正後も単独参加と不戦勝を含む4件が失敗した。judge_gameは単独参加を生存者勝利にしないようにし、runは返された勝者にwith_winを適用することで、同じ期待値の13件すべてが成功した。
- テスト登録：作業中に旧登録ファイルが削除されたため、その削除を維持して新しい登録へ整理する方針を提示して進めた。core_models.rsは実ソースのモデルテスト49件、application.rsはドメインサービス31件・CUI31件・ゲームループ13件を登録する。旧cui.rsを指すCargo.tomlの明示登録は除去した。
- 旧Strategyへの委譲回数など廃止されたAPIのテストは、実装型をtrait越しに呼び出すaim/hitの契約へ書き換えた。個体ID・順序・不変性・借用終了後の所有は引き続き検証している。テストの無効化・空のテスト・期待値の緩和は行っていない。
- 検証：`cd rust_molkky && cargo test --locked`で全124件成功。失敗・無視は0件。勝利反映・単独参加・得点履歴を含め、直前のCUI再配置時に残っていた5件も解消した。
- Java：JDK 17を指定した`./gradlew test --console=plain`が成功。JUnit XMLを集計して全28件成功・失敗0・エラー0・スキップ0を確認した。JavaのコードとGradle設定は変更不要だった。
- 旧登録ファイルの削除は新しい登録への移行としてコミットへ含める。別途削除されたgame・simulation・view・tuiのテスト本体と旧API文書は、この修正のコミットへ取り込まない。
- 未確認：恒久的なJAVA_HOME設定とエディター上の検証は行っていない。今回のJava実行では一時配置のJDKを指定した。
- 最終確認：`cargo build --locked`、対象Rustファイルのrustfmtチェック、`git diff --check`が成功した。

## CuiStickとthrowによる確率的な転倒判定（2026-10-30）

- 対象：Molkkyのthrow抽象メソッド、CuiMolkky、CuiStick、ゲームループ・起動元と対応テスト。既存の別途削除中のファイルは変更しない。
- 期待する振る舞い：aimで選んだ各スキットルを、棒の確率で独立に転倒判定する。hitは転倒した個体だけを返し、ID・順序と元のFieldを保持する。確率の指定がない通常起動では50%を採用する。これはこのCUIの取り決めであり、モルックの公式ルールから導いた値ではない。
- 設計：Molkkyに関連型Stickと必須メソッド`throw(&self, stick: &Self::Stick) -> f64`を追加する。aim・hitも関連型の棒を受け取る。CuiStickは既存のStickと0〜1の確率を保持し、CuiMolkky::throwはその値を返す。coreからCUI型を参照しない。
- 入力は倒れた番号から狙う番号へ意味を変更する。runへCuiStickを渡し、通常起動では50%、既存の得点ルールの検証では100%を指定する。乱数源やStrategyを新たに注入するAPIは追加しない。
- [x] CuiStickの識別子委譲、確率の保持・境界・範囲外を検証する。
- [x] throwが指定した棒の確率を返し、0%では0本、100%では選択した全個体だけが転倒することを確認する。
- [x] 既存の採点・終了・不変性・個体IDのテストを100%の棒で維持し、0%のゲームループも検証する。
- [x] スキットルを扱うイテレータのpinをskittleへ改名し、呼び出し元・表示・READMEを更新する。
- [x] 全Rustテスト・ビルド・整形・差分を確認する。
- 変更前：`cargo test --locked`は124件成功。

### Red・Green・Refactorと結果

- Red：`cargo test --locked --test application molkky::probability::`で、throwが固定値0.5を返しhitが全個体を倒す最小実装に対し、棒の確率の取得と0%の転倒判定の2件が期待した理由で失敗した。throwからhitへの確率受け渡しをまとめて確認するため、この2件を同時に選択した。
- Green：throwはCuiStickの確率を返し、hitは一度取得した確率で各スキットルに`rand::random_bool(probability)`を適用する。既存のrand 0.10.3のAPIをローカルのソースで確認して利用し、新しい依存関係は追加しない。
- 確率0%と100%を使う検証は毎回同じ結果になる。乱数の頻度を数える不安定な合否判定は追加していない。CLIの既定50%の検証は、入力5・8に対して仕様上可能な0・5・8・2点のいずれかと1投の記録を確認する。従来の厳密な得点・勝敗の期待値は100%のゲームループで引き続き検証する。
- Refactor：スキットルのイテレータ変数をskittle、一覧をskittlesへ統一した。AGENTS.md内のコード例も同じ命名へそろえた。入力案内とREADMEは狙う番号の選択として更新した。
- 公開APIの変更：Molkkyの実装は関連型Stickとaim・throw・hitを定義する。traitオブジェクトは`dyn Molkky<Stick = CuiStick>`とする。runの第4引数へ`&CuiStick`を渡す。mainと既存の全呼び出し元を更新した。
- 検証：`cargo test --manifest-path rust_molkky/Cargo.toml --locked`で全130件成功（application 81件・core_models 49件）。失敗・無視は0件。`cargo build --locked`、対象ファイルのrustfmtチェック、git diff --checkも成功した。ソース・登録済みテスト・AGENTS.mdのコード例にpin/pinsの変数名が残っていないことを検索で確認した。
- 未実施：Javaの再検証、確率分布の統計的検証、エディター上の確認。Java側の変更はない。
