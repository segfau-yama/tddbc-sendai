# 開発支援方針の記録

## 衝突通知でhitとchain_hitを呼ぶ設計の評価（2026-11-05）

- 対象：7951109のservices/molkky.rs:L20・L67・L98・L172・L217、services/game.rs:L101、views/stick.rs:L260と対応テスト。ユーザー案はviewsのオブザーバーから衝突種別に応じてhit/chain_hitを呼び、GuiMolkky::updateを削除するもの。今回は設計評価で、ソースとテストは変更しない。
- P2：実際に衝突した棒とスキットルへhit、スキットル同士へchain_hitを呼ぶ案に賛成する。現状の全スキットルへのhitと全組み合わせのchain_hitは、Rapierの接触ペアを種別で振り分ける形へ限定できる。スキットル同士の接触ペアには転倒元→先という順序がないため、両個体に必要な状態反映を行う。
- P1：衝突検知の根拠はRapierの接触情報に置く。viewsの座標やSVGの重なりを監視すると、描画間の高速衝突や寸法・回転を物理と別に判定することになる。描画側は既存のSignal<Game>を読み取っており、状態変更からの再描画に独自Observer型や購読機構を追加する必要はない。衝突をviewのuse_effectでゲーム更新へ変換する構成も採用しない。
- updateの分解：現在は保持/初速のボディ反映、固定刻みの共有world.step、接触力積の蓄積、棒と全個体の位置/速度/睡眠状態の反映、hit/chain_hitの振り分けを持つ。衝突通知で置き換えられるのは振り分けだけであり、残りの処理は衝突しない刻みでも必要。GuiMolkky::updateというメソッドは物理処理の所有と実行をphysics側へ移す場合に削除できるが、処理を消すことはできない。
- 削除する場合の分担案：物理側は一刻みの全物体更新・接触/力積・sleep結果、GuiMolkkyはaim/hit/chain_hit/stand_upの具象化、Gameは結果のモデル反映と投擲完了/採点/手番への接続、viewsは入力・更新要求・モデルの描画を担当する。物理ワールドを個別のviewやモデルへ重複所有させない。専用Observer traitやイベントバスは作らず、接触一覧を直接処理するところから始める。
- API上の注意：現行hit/chain_hitは引数の個体だけでなくself.world/self.impactsを読み、位置とHit/Fall/Standを返す。ワールドを別オブジェクトへ移す場合は、物理結果をGuiMolkkyがどのように参照するかも決める必要がある。関数名の移動だけでは分離できず、coreへRapier型・座標を加える案は採らない。今回、未確定の新しい型・traitの契約は実装しない。
- 維持条件：接触開始時にHitとなった後、接触がなくても動いている間はモデルを更新し、停止時に力積からFall/Standを確定する。接触開始イベントだけでは完結しない。同じ物理刻みを全体で一度進め、全接触の結果を反映してからStopかつHitなしを確認し、一投だけ採点する。表示の回数や部品数で進行を変えない。
- ユーザー追記への回答：状態を決める責務はhit/chain_hitへ集約する。現行のhit（molkky.rs:L188）とchain_hit（L241）は既にHit/Fall/Standを判断し、同じIdの更新後GuiSkittleを返している。updateはその結果を呼び出し元へ返す接続を担い、状態判定を別に再実装する必要はない。
- 呼び出し時期は分けて考える。接触時にStand→Hit、停止時に力積からHit→Fall/Standを返す既存の意味を維持するなら、接触通知に加えて停止時にも該当するhit/chain_hitを再評価する。接触履歴を使えば停止時に新しい接触がなくても判断できる。衝突時に直ちにFallへ確定する案は、弱い衝突とHit継続中の採点防止という既存契約の変更になるため、今回の追記からは採用しない。
- [x] 現行の実行順とワールド所有、hit/chain_hitの依存、固定刻み・弱い衝突・連鎖・Hit継続時の採点防止テストを静的に確認した。
- [x] 利点・残る責務・削除条件・契約の未確定部分を記録した。設計評価のみなので動作テストは再実行せず、git diff --checkで文書を検証する。記録だけをcommit・通常pushする。

## Stickコンポーネントの入力と投擲状態（2026-11-05）

- 対象：views/stick.rs・mouse.rs・app.rs、GUIの入力接続と対応テスト。StickをDioxusコンポーネントにし、mouseの操作・DOM接続・経過時間の処理を同じファイルへ集約する。共有するSignal<Game>を受け取り、GameやGuiStickを重複所有しない。appは画面合成と共有Gameの用意を担当する。
- 状態：有効な左押下でHeld、押下中は目標とspeedだけを変えてHeldを保ち実速度・位置を変えない。解放時の目標と表示速度で既存throwを一度呼びMoveへ進む。以降は固定刻みでサービスを更新し、GuiStick::update→moveから速度とMove/Stopを受け取る。物理時間を各部品で重複更新しない。
- 受付：投擲中・終了後・他ポインター・右ボタンは開始しない。領域外解放は最後の有効目標を使い、取消・focus喪失は投げずにHeldへ戻す。無効な方向や0速度は従来どおり投げない。StopかつHitなしの採点条件、採点/手番/勝敗のcore利用を維持する。
- [x] 変更前のGUIテストを確認する。
- [x] Stickコンポーネントの描画APIと押下→蓄力→解放→減速の契約をテストしRedを確認する。
- [x] mouseをStickへ統合し、コンポーネントが入力・DOM接続・破棄を持つよう移行する。
- [x] Heldの正速度でも移動/採点しないこと、解放速度の一致、投擲中の減速を確認する。
- [x] 座標変換・capture・取消・所有ポインター・二重加点防止・次投の既存検証を維持する。
- [x] 全テスト・fmt/Clippy・feature分離・Web/desktopビルドとWeb実操作・差分を確認する。
- [x] 検証結果を記録しConventional Commitで通常pushする。

- Red：先に既存の描画テストをSignal<Game>を渡すStickコンポーネントへ変更し、views::Stickが未定義で失敗することを確認した（/tmp/molkky-stick-component-red.log）。APIのRedであり、既存の投擲ルール自体が失敗した記録とは区別する。
- Green/Refactor：変更前の205件を確認後、マウス入力とappの入力接続をStickInputへ統合した。Stickコンポーネントがuse_signal・document::eval・use_dropを持ち、Appが作る一つのSignal<Game>を共有する。mouse.rsを削除し、JS接続と破棄テストはstick.jsへ移動した。Appは共有Gameの生成・画面合成・タイトルを保つ。採点・手番・物理サービスは変更していない。
- moveの責務：GuiStick::updateは物理結果を渡し、既存moveが実速度ベクトルからspeedとMove/Stopを更新する。Heldのmoveは蓄力値をそのまま保つ。減衰や位置積分をコンポーネントへ重複実装していない。
- テスト移行：旧mouseの座標・所有ポインター・解放・取消、旧appの入力/物理接続と棒の描画をtests/gui_2d/views/stick.rsへまとめ、レイアウトと表示範囲はapp.rsへ残した。SSRは実際のStickコンポーネントをVirtualDomで構築する。状態属性と照準の表示差だけを区別し、次投の位置・形状・識別子の検証を維持した。テスト用の共有fixtureは同ファイル内に置き、製品のラッパーや新たな依存は追加していない。
- 追加確認：押下→0.5/1/1.5秒の蓄力でHeld・実速度0・保持位置・同じId・未採点を確認するテストが成功した。解放時に表示どおり5m/sでMoveへ進み、一刻み後には正の速度のまま減速し、後続のMove/Releaseイベントで投擲速度を再設定しない。このテストは既存契約の維持を確認するもので、追加時から成功したためRedとは扱わない。
- 検証：cargo test --locked --features gui_2dは206件、cargo test --lockedは149件、GUIのみのcargo testは113件が成功し、失敗・無視0件。ログは/tmp/molkky-stick-component-green.log・default.log・gui-only.log。Stick対象18件も成功した（/tmp/molkky-stick-component-focused.log）。cargo fmt --check、cargo clippy --locked --all-targets --features gui_2d -- -D warnings、coreのみのcargo check、Webビルド、desktopのcargo check、git diff --checkが成功した。desktopは既存の/workspace/.cloud-tools/desktopをPKG_CONFIG_SYSROOT_DIR/PKG_CONFIG_PATHで参照した。
- 実操作と寿命：node tests/gui_2d/views/stick.jsの通常終了/接続失敗時の破棄2件が成功した。Chromiumで押下中のHeld・正速度でも同じ位置・解放後のMoveを追加確認し、capture・取消・領域外解放・リサイズ・連続投擲・採点・失格・結果・反射も成功した（/tmp/molkky-stick-component-browser.log）。検証用8081番サーバーは終了した。デスクトップはコンパイルのみで、実機操作は未確認。

## mouseの責務をStickのviewへまとめる設計レビュー（2026-11-05）

- 対象：a322222のgui_2d/views/mouse.rs・stick.rs・app.rs、models/stick.rs、services/game.rsと対応テスト。リモートの最新構成ではcontrollersが削除され、入力はviews、ゲーム進行はservices/Gameへ移っている。この構成を前提に評価し、controllersの再導入は提案しない。今回はレビューのみで、ソース・テストは変更しない。
- P2 / shrink：views/mouse.rs:L64のMouseInputは棒の照準・蓄力・解放・取消だけに使われている。一方、views/stick.rs:L15は棒を描画するだけなので、一連の棒の操作を追うにはapp.rs・mouse.rs・stick.rsを往復する。mouse.rsの既存処理をviews/stick.rsへ移し、MouseInputをStickInputとする案に賛成する。元のMouseInputを残して委譲するラッパーや、新しいcontroller層は追加しない。
- 責務の境界：views/stick.rsはポインターID・座標変換・経過時間による蓄力・入力取消と棒/照準の描画を担当する。models/stick.rsは目標・位置・速度・coreの状態とnew/throw/move/updateを担当する。マウスボタン、DOM矩形、フォーカス喪失はviewの入力境界へ残す。coreにも持ち込まない。
- views/app.rsはイベントの配送・全体の描画合成・フレーム時刻の管理を担当し、ゲームの受付条件や投擲は既存Gameへ接続する。StickInputが別のGuiStickやGameを所有せず、サービスが保持する個体を描画する。Game::aimは既にGuiStickの準備と物理照準を集約し、Game::throwはGuiStick::throwを使っているため、同じ準備や状態遷移をviewへ再実装しない。
- 維持する検証：座標の拡縮・余白・上下反転、所有ポインター、領域外の解放、表示と同じ初速、取消時に投げないこと、投擲中/終了後の受付停止。実装時はtests/gui_2d/views/mouse.rsをstick.rsへ統合して、対象ファイル名を一致させる。
- [x] 最新の入力・描画・モデル・ゲームサービスの経路と対応テストを静的に確認し、根拠・優先度・移動先を記録した。
- [x] 文書だけの変更なので動作テストは再実行せず、git diff --checkで確認する。リモートの変更を保持してレビュー記録のみcommit・通常pushする。
- net: -0 lines possible（単純な移動・改名の場合）。ファイル数は1つ減らせる。未実装のコンポーネント再編による行数削減は見積もらない。

## controllersを撤去しviewsのコンポーネントとservicesへ移行（2026-11-04）

- 対象：gui_2dのcontrollers・view・app、GUIモデルとサービス、対応するテスト・起動元・README。最新の指示を優先し、前節のcontrollerを残す案は置き換える。
- 設計：views/Appが入力・フレーム時刻・リアクティブな描画とDOM接続の破棄を所有する。services/Gameが投擲・採点・終了・交代、GuiMolkkyが共有ワールドと衝突を担当する。GuiFieldは個体一覧を持たず、GameがGuiSkittleの一覧を保持する。既存coreの得点・勝敗ルールを再利用する。
- API：GuiStick::updateは位置・角度・速度・角速度から同じIdの更新値を返し、既存Moveの判定を使う。GuiSkittle::updateは位置・角度・判定済み状態・照準状態から同じId/番号の更新値を返す。共有ワールドは固定刻みごとに1回だけ進める。modelsの非公開フィールドを保ち、core/CUIや依存関係は変更しない。
- 完了条件：controllersを削除し、views内で描画・入力・更新・破棄を扱えること。現在の固定フィールド・反射・連鎖・入力取消・採点・勝敗を保つこと。作業開始時の既存コンパイル不整合はTDDのRedとは数えない。
- [x] モデルのupdateがId・投擲情報・元の値を保ち、位置・速度・停止状態を更新するテストを先に実行した。
- [x] GuiSkittleのupdateと標準配置、独立したGuiFieldをテストした。
- [x] 物理サービスをモデルのupdate経由に変更し、一刻み・衝突・壁・連鎖の既存検証を維持した。
- [x] ゲームサービスに採点・勝敗・立て直し・交代を移し、Hitが残る間の採点防止と二重加点防止を確認した。
- [x] viewsコンポーネントへ入力・フレーム・描画・破棄を移し、controllersを削除した。
- [x] テストの配置と登録・公開API・READMEを更新し、GUI/CUI・feature分離・fmt/Clippy・Web実操作・差分を確認した。

### 実装と検証の記録

- Red：GuiStickの物理結果反映と停止条件の2件は、同じupdate APIの契約としてまとめて選び、update未定義によるE0599を確認した（/tmp/molkky-components-model-red.log）。実装後は2件成功。次にGuiSkittleのupdateを1件選び、未定義で失敗させてから実装し成功した（/tmp/molkky-components-skittle-red.log）。モデルのみの6件が成功した。これはAPIのRedであり、物理の振る舞いを実行して失敗した記録とは区別する。
- Red→Green：JSの破棄テストを先に追加した。通常終了は成功したが、接続失敗時はcaptureが残るため失敗した（/tmp/molkky-components-dispose-red.log）。views/app.jsの後始末をfinallyに置き、イベントリスナー・requestAnimationFrame・pointer captureを必ず解除して2件とも成功した（/tmp/molkky-components-dispose-green.log）。追加のテスト依存は不要で、Node.js標準ライブラリを使う。
- Refactor：controllersを撤去し、views/Appコンポーネントにuse_signal・入力・フレーム時刻・描画合成・use_dropを集約した。既存のElementを返す描画部品はviewsへ移して再利用する。Phaseは入力の所有と棒の状態から求め、重複して保存しない。サービスやモデルはDioxusに依存しない。
- services/Gameは注入したチーム・フィールド・棒・個体一覧を保持し、照準・投擲・完了時の採点・勝敗・立て直し・交代を接続する。coreのルールを複製しない。GuiMolkkyは共有ワールドを進め、物理結果をGuiStick::updateとGuiSkittle::updateで反映する。個体数0本と12本で棒の一刻みの移動量が同じになる退行テストも成功した。このテストは追加時から成功しており、Redとは扱わない。
- API移行：create_fieldはGuiField・GuiStick・Vec<GuiSkittle>を返し、create_worldは既存モデルから物理を構築する。GuiMolkky::updateは個体一覧と棒を受け、更新後の一覧と棒を返す。gui_2d::views::Appを起動元とし、旧app/controllers/viewへの公開経路を更新した。GuiFieldの個体保持を復活させず、モデルのフィールドも非公開のまま保つ。
- テスト移行：controllersの採点・初期化・Hit継続時の完了判定はtests/gui_2d/services/game.rsへ、入力・表示・固定範囲はtests/gui_2d/views/app.rsへ移した。Fieldの識別子はGameでも、同番号の別個体と位置保持は物理テストでも検証を維持する。旧APIのため成立しなくなったテストを無視したり、期待値を実装に合わせて弱めたりしていない。
- 最終検証：`cd rust_molkky && cargo test --locked`は149件、`cargo test --locked --features gui_2d`は205件、`cargo test --locked --no-default-features --features gui_2d`は112件成功し、失敗・無視0件。ログは/tmp/molkky-components-default.log、/tmp/molkky-components-green.log、/tmp/molkky-components-gui-only.log。`node tests/gui_2d/views/app.js`も2件成功した。
- 品質・ビルド：`cargo fmt --check`、`cargo clippy --locked --all-targets --features gui_2d -- -D warnings`、`cargo build --locked --no-default-features --features gui_2d_web --bin gui_2d --target wasm32-unknown-unknown`、git diff --checkが成功した。core/CUI・Cargo.toml/Cargo.lockは変更していない。
- Web実操作：検証専用のdxサーバーを127.0.0.1:8081で起動し、既存のChromiumテストtests/gui_2d/app.jsで入力・capture・取消・連続投擲・採点・失格・結果・固定範囲・反射が成功した（/tmp/molkky-components-browser.log）。Playwrightは検証用に/tmpへ配置し、製品の依存には加えていない。
- 検証後、実行引数で特定した8081番サーバーだけを停止した。文書差分を確認し、今回の関連ソース・テスト・READMEと実装記録をコミットする。作業開始前からの過去の開発記録の削除は作業ツリーに保つ。
- 未確認：デスクトップ版のコンパイルと実機操作。現在の環境ではxdoとayatana-appindicatorの開発情報が見つからず、Webを検証対象とした。実装したゲームの範囲に未完了の項目はない。今回は新しいSTEPのルールを追加していない。

## モデルの物理状態更新とcontrollerの画面更新を分ける案（2026-11-04）

- 状態：最新の依頼によりcontrollerを残す案を置き換え、個体のupdateを実装し、画面更新をviewsのコンポーネントへ移行した。上の実装記録を参照する。
- 対象・入力：前節P1に対するユーザー案。skittle/stickモデルに物理計算のupdate、controllerに画面のupdateを置く案を、現在のRapier共有ワールド・入力処理・既存テストと照合する。今回は設計の検討であり、実装の指示とは扱わない。
- [x] 個体の状態更新と共有物理ワールドの時間進行を区別した。
- [x] controllerのフレーム更新とviewの描画を区別した。
- [x] 責務・不変条件・判断理由と未実装事項を記録し、git diff --checkで文書差分を確認した。

| 担当 | updateに持たせる責務 | 境界 |
| --- | --- | --- |
| models/stick.rs | 自分の位置・角度・速度などの物理結果を反映した新しいGuiStickを返す。Move/Stopは既存のMoveの判定を利用する。 | Idと投擲情報を保ち、他の個体や画面を操作しない。 |
| models/skittle.rs | 自分の位置・角度と、hit/chain_hitで判定した状態を反映した新しいGuiSkittleを返す。 | Id・番号を保つ。相手との衝突・連鎖をモデル単体へ移さない。 |
| services | 共有ワールドを固定刻みごとに1回進め、接触結果を使うaim/hit/chain_hitなどの相互作用を接続する。物理結果の反映にはモデルのupdateを使う。 | 同一刻みの全個体を同じ物理計算結果から更新する。実際に必要な更新・判定だけを分け、専用traitや中間型を前提にしない。 |
| controllers | 入力と経過時間を受け、必要な固定刻みを実行し、更新したモデル・採点・手番を反映してviewへ渡す。 | 「画面更新」は進行と再描画の制御を意味する。HTML/SVGの生成はviewが担当する。 |
| view | 渡されたモデル・進行状態・判定済み結果を描画する。 | 物理時間を進めず、採点・勝敗・衝突を判定しない。 |

- 判断：個体自身のupdateをモデルへ置く案に賛成する。非公開フィールドをサービスが直接構築する問題にも、モデル自身の更新APIで対応できる。一方、Rapierは棒・スキットル・壁の衝突を共有ワールドで計算する。各モデルのupdate内でworld.step()を実行すると、標準12本と棒で同じ刻みに13回進み、個体数に応じて速度や衝突結果が変わる。各モデルが別々に位置を積分する形も、現在のRapierによる移動計算と二重にならないようにする必要がある。
- 更新の順序：controllerが経過時間を蓄積する → 必要な固定刻みごとにサービスが共有ワールドを1回進める → 接触と各ボディの結果から、モデルのupdateとhit/chain_hitを使って各個体を更新する → controllerが投擲完了時だけ採点・交代を反映する → viewへ更新したモデルを渡す。棒がStopでもHitが残れば採点しないという既存条件は維持する。
- 前節P1との関係：必要なのは型・構築・更新の定義と呼び出し元を揃えること。すべての更新をGuiMolkky::updateへ集約する配置を必須とはしない。この案では個体の状態更新をmodelsへ分け、共有物理更新をservicesへ残す。
- 根拠：HEADのservices/molkky.rsのupdateはworld.step()を1回実行し、各ボディの結果をモデルへ反映していた。作業ツリーでは削除中。tests/gui_2d/services/molkky.rs::movementは一刻みの移動・初速の再適用防止、tests/gui_2d/controllers/app.rsはStopとHit解消後の採点を検証する。後の実装でもこれらの期待値を維持する。
- 未実装・検証：今回はRust/JSとテストを変更せず、既存のコンパイル不整合も未解消。文書のみの検討なのでRed/Green/Refactorと動作テストは実行しない。前節のコンパイル失敗を再びRedとして記録しない。

## gui_2dの新しい設計指針との整合レビュー（2026-11-04）

- 状態：当時の作業ツリーに対するレビュー。最新の依頼でcontrollersの撤去とモデル更新・サービス構築の移行を実装した。コンパイル失敗の記録は当時の結果として残し、現在の検証結果は冒頭へ記録する。
- 対象：作業ツリーの `rust_molkky/src/gui_2d` と対応テスト・呼び出し元。提示されたviews/controllers/models/servicesの責務と、修正済みmodelsを前提に確認する。
- 完了条件：依存関係と実行経路を追い、指針との不一致を根拠・具体例・優先度付きで記録する。今回はレビューのみで、ソースとテストは変更しない。
- 前提：1画面で1試合を動かす現在のGUIを対象とする。viewの条件分岐・座標変換など描画に必要な処理と、勝敗・採点・衝突などの判断を区別する。`views` は描画層の指定として読み、`view->view` の表記だけから改名先を断定しない。
- [x] viewsとcontrollersの依存・描画処理を確認した。
- [x] modelsを元にしたサービス構築とモデル間の相互作用を確認した。
- [x] 修正済みmodelsと呼び出し元の整合をコンパイルと既存テストの内容で確認した。コンパイルは失敗しており、動作確認済みとは扱わない。
- [x] 優先度・判断理由・最小の修正案と未確認事項を記録し、文書差分を確認した。

### 指針ごとの判定

| 層 | 判定 | 根拠 |
| --- | --- | --- |
| views（現行名はview/） | 概ね整合 | view/以下はモデルの状態・座標・速度・判定済み勝者をHTML/SVGへ変換する。採点・衝突・勝者選択は行っていない。ただし画面全体とフィールド内の描画がcontrollersに残る。 |
| controllers | 部分的に整合 | Controllerのフィールドにviewはなく、MouseInputによる入力制御、サービス呼び出しとモデル更新の接続は役割に合う。一方、controllers/app.rsとfield.rs自身がElementを生成している。 |
| modelsを元にした構築 | 移行未完了 | GuiFieldから個体一覧が外れ、GuiStick/GuiSkittleが非公開フィールドになったが、呼び出し元とテストは旧API・直接構築に依存したまま。 |
| services | 一部不一致 | aim/hit/chain_hitは棒とスキットルの相互作用を扱っている。一方、チーム名からの生成、標準配置の用意、物理エンジンの構築も同じサービス群に混在する。 |

### 指摘

1. **P1：GuiMolkkyの実装を支える定義と更新処理が欠けている。**
   - 対象：src/gui_2d/services/molkky.rs:L6、services/mod.rs:L4、services/field.rs:L1、controllers/app.rs:L91。
   - 根拠：GuiMolkkyの構造体、TIME_STEP/DAMPING/FALL_IMPULSE、updateが削除されているが、Molkkyのimpl、公開use、ワールド構築、Press/Move/Frameからのupdate呼び出しは残る。現在のcargo checkは型と定数の未定義を報告する。
   - 影響：GUIのライブラリ自体がコンパイルできず、描画や投擲を確認できない。
   - 修正案：修正済みモデルを受け取る具象サービスの構築・更新経路と必要な定義を揃える。旧実装をそのまま戻すだけでは、次項のAPI不整合が残る。

2. **P1：修正済みmodelsを利用するための保持先・更新経路が揃っていない。**
   - 対象：models/field.rs:L6、controllers/app.rs:L37・L161・L210、controllers/field.rs:L10、services/field.rs:L27、services/molkky.rs:L49・L96・L141・L163。
   - 根拠となる入力と結果：GuiField::newはFieldだけを受け、skittles()も持たない。利用側は2引数のnewとfield.skittles()を使い、Controllerにも独立したVec<GuiSkittle>がない。そのため一覧を取得して転倒確認・採点・立て直し・描画する経路が成立しない。cargo checkでE0061/E0599を確認した。
   - 追加の静的指摘：GuiSkittleのフィールドはmodels/skittleモジュールだけが直接扱えるが、サービスは構造体リテラルでposition/angle/aimed/skittleを書き換えている。公開APIには角度・照準状態を保持した更新値を作る操作がなく、コンストラクタは角度0・aimed=falseにする。現在のコンパイラ診断は前項の未定義型で止まるため、この非公開フィールドの問題は静的な指摘として区別する。controllers/app.rsの既存テストにもGuiStickの直接構築が残る。
   - 修正案：現在のGuiFieldの責務を前提に、個体一覧をControllerなどの試合を組み立てる側で保持し、同じ個体をサービスとviewへ渡す。モデルが非公開状態を保ったまま更新後の値を作れる最小APIを揃え、サービスとテストもそのAPIへ移行する。

3. **P2：controllersが描画そのものを持っている。**
   - 対象：controllers/app.rs:L227、controllers/field.rs:L8。
   - 根拠：appのrenderはstyle・見出し・画面レイアウト・得点表・結果をrsx!で構築し、fieldのrenderはg要素・照準の線・棒を組み立てている。現在は得点表の配置や照準ハイライトを変えるだけでもcontrollersの編集が必要になる。
   - 修正案：この2つの描画処理をviewへ移す。Controllerは進行とサービス呼び出しを担当し、モデル・進行状態・判定済み勝者をviewに渡す。移動先のviewがControllerやGuiMolkkyを参照して勝敗判定する形にはしない。既存のcontrollers/app.rs::compositionなどの描画テストも対応するviewのテストへ移す。
   - 判断：「viewを持たない」はviewのインスタンスや描画実装を所有しないという意味で評価した。controllerからviewを呼んでモデルを渡す依存自体は、viewsを制御するという指針と整合する。

4. **P2：servicesにモデルの初期化と物理ワールドの構築が混在している。**
   - 対象：services/teams.rs:L3、services/field.rs:L10・L34。
   - 根拠：from_namesはNameとTeamを生成するだけで、チーム間のルールを扱わない。create_fieldは固定の標準配置・棒・サービスをまとめて生成する。create_worldはモデルからRapierのボディ・壁・Id対応を作る構築処理であり、hitやchain_hitのような個体間の判断とは役割が異なる。たとえば初期配置だけを変える際にも、ドメインサービス群の編集になる。
   - 修正案：標準モデルの用意は起動時の初期化へ寄せる。モデル一覧からのワールド生成はGuiMolkkyの構築へ寄せ、servicesの操作はモデル間の相互作用へ集中させる。汎用Factoryや新しいPhysics traitを追加する必要はない。Rapier専用処理を独立させる場合も、まず必要な構築処理の範囲だけにする。

### 整合している処理と判断の境界

- view/score.rsはtotal/state/historyを参照して文言へ変換するだけで、得点や失格を再計算しない。view/result.rsも判定済みのOption<&Team>を受け取る。色の選択、角度の度への変換、SVGのY反転、文字列化、表示・非表示の分岐は描画の責務として残せる。「ロジックを持たない」を計算・分岐が一切ないこととは解釈しない。
- controllers/mouse.rsの座標・時刻変換、pointerの所有、蓄力・取消は入力制御に合う。これらをドメインサービスへ移す理由はない。
- finish_throwとupdate_game_stateはcoreのcalc_score、ScoreBoard.add、judge_game、next_teamを呼び、結果をモデルに反映している。この接続順序はアプリケーションの進行であり、処理を丸ごとドメインサービスへ移す必要はない。勝敗のルール自体は引き続きcoreのjudge_gameが担当する。
- GuiTeam/GuiScoreBoardは宣言・変換・再公開だけで、Controllerとviewはcore::Team/ScoreBoardを使っている。これは利用状況として記録する。coreのモデルもドメインモデルなので、Gui型を全箇所で使わないことだけを責務違反とはしない。GUI独自の意味がないラッパーを使うためだけの変換は増やさない。
- `view->view` は同名の表記のため、ディレクトリの改名先はこの入力から確定できない。現在はview/、controller側はcontrollers/。今回、名称の変更は行っていない。

### 検証と残作業

- 実行：`cd rust_molkky && cargo check --locked --no-default-features --features gui_2d`。終了コード101、16件のコンパイルエラー・3件の警告。ログ：/tmp/molkky-design-guidelines-check.log。今回のレビューで見つけた既存作業ツリーの不整合であり、TDDのRed確認としては記録しない。
- テストの静的確認：models/field.rs、services/field.rs・molkky.rs、controllers/app.rsの既存テストも旧GuiFieldのAPIを利用する。描画テストではモデルを元にした表示、得点表の状態参照、HTMLエスケープ、固定範囲などを確認している。
- 未確認：コンパイル失敗のためRustテスト・Web/desktopの実行確認は行わない。レビューではソースとテストを修正していない。Red/Green/Refactorの実装工程も対象外。
- 修正順：1・2の構築とモデル利用を揃え、3の描画移動と4の初期化整理を行う。最後に既存の入力・描画・採点・連鎖・壁・手番のテストで振る舞いを確認する。指摘の修正は今回未実装。削減行数は移動中心のため算定しない。
- 文書確認：git diff --checkで余分な末尾空行を見つけて除去し、再確認で成功した。今回のレビュー記録だけをコミット対象にし、作業開始前からのソース変更と過去の開発記録の削除は作業ツリーに保つ。
