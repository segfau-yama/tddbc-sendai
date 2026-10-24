# tddbc-sendai
TDDBC仙台の備忘録用リポジトリ

## Java演習環境

- JDK 17が必要です（JREのみではコンパイルできません）。
- 必要に応じて、`JAVA_HOME`をJDK 17のインストール先に設定してください。
- GradleはWrapper経由で取得するため、事前インストールは不要です。
- 初回実行時は、GradleとJUnitの取得にインターネット接続が必要です。
- 実装は`src/main/java`、JUnit 5のテストは`src/test/java`へ配置します。

```sh
./gradlew test        # テストを実行
./gradlew build       # テストとビルドを実行
./gradlew clean       # ビルド結果を削除
```

Windowsでは`./gradlew`の代わりに`gradlew.bat`を使用します。
初期状態には演習用コードやテストケースを含めていません。
