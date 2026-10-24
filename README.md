# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Rust演習環境

- `rust_molkky`は、Java版の実装とテストを移したライブラリです。
- 実装は`rust_molkky/src/`に構造体ごとに分け、テストは`rust_molkky/tests/`にあります。外部依存はありません。
- Java版のSkittle・Throw・Scoreに対応し、STEP 01・02を実装しています。
- `Skittle::new`が番号1〜12を保証し、`Throw::new`が番号の重複を拒否します。
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
