# TDDBCの開発支援方針

- このリポジトリは、TDDを実習するための学習用プロジェクトである。
- [STUDY_AI.md](STUDY_AI.md)の要点と、このリポジトリ独自の運用規則を、AIの作業指針として整理している。
- 講演の提案や将来予測は、確立した定義・事実と区別する。
- `mod`での階層化、Rustの配置・検証コマンド、自動コミット・push等は独自の運用規則であり、講演で示された必須事項として扱わない。

## 仕様と作業範囲

- 実装前に、期待する振る舞い・制約・完了条件を整理する。
- 演習では、公開済みの仕様を対象にする。先の課題を作り込まない。
- 曖昧な点や未確認の前提を明示し、判断理由を残す。
- 条件が複雑な場合は、EARS・Gherkin・デシジョンテーブルなどで整理する。
- 守りたい品質・制約・機能要求から、必要な設計を決める。

## TDDの進め方

- まず、確認したい振る舞いをテストリストにまとめる。
- 原則として、リストからひとつ選び、実行可能なテストを書く。
- 講演で提案された複数選択を使う場合は、対象とまとめる理由を明示する。
- Red：実装前にテストを実行し、狙った理由で失敗することを確認する。
- Green：必要な実装を加え、新しいテストと既存テストを成功させる。
- Refactor：成功を保ちながら、必要な範囲で設計を改善する。
- 作業中の気づきはテストリストへ追加し、完了した項目を消し込む。
- 振る舞いを変えないリファクタリングでは、既存テストで変更前後を確認する。

## テストと品質

- テストスイートを、仕様を表す第一級の成果物として扱う。
- 期待値は仕様から決め、実装の出力をそのまま正解にしない。
- 成功させるために、検証の削除やテストの無効化を行わない。
- 空のテストや無視されたテストを、検証済みとして扱わない。
- リスクと影響度に応じて、境界値・異常系・退行を確認する。
- 検証手法や依存関係は、目的に必要なものだけ追加する。
- Rustのテスト本体は`tests/`以下へ置き、対象のファイル名と一致させる（例：`src/tui/team.rs`に対して`tests/tui/team.rs`）。テストを登録するmod宣言やエントリーファイルにはテスト本体を置かない。
- Rustのテストケースは、検証する振る舞い・前提条件など意味のある単位で`mod`にまとめ、入れ子の`mod`で階層化して体系的に整理する。JUnitの`@Nested`と同様のグループ化の目的で用いる。

## コードのコメント

- プロダクトコードの重要な条件判定には、業務ルールや判断理由を短くコメントする。
- コード自体で意味が明確な処理には、説明を繰り返すコメントを付けない。

## 自律的な作業とレビュー

- 調査・変更・実行・結果確認を繰り返し、失敗原因を確かめて修正する。
- 変更を小さく保ち、レビュー対象と範囲を明確にする。
- コード生成量を成果とせず、確認可能な変更を積み重ねる。
- テスト結果と差分を確認し、目標以外の制約も守られているか点検する。
- コミットする場合は、Conventional Commitsで変更の目的を示す。
- 依頼された変更は、関連する検証と差分確認を終えたら、コミット・指定先への通常のpushまで自動で行う。ユーザーが保留や別の手順を明示した場合はその指示を優先する。
- Red・Greenなどの段階を分けて記録し、作業履歴を追いやすくする。

## 理解と意図を残す

- 技術負債に加え、認知負債と意図負債を意識する。
- 変更理由・仕様上の判断・制約を、テストや文書に短く残す。
- 利用者が説明できるよう、実装の根拠とトレードオフを伝える。
- 不明点を理解済みとして扱わず、必要な知識をペアで共有する。
- 完了時は、変更内容・検証結果・未実装事項を簡潔に報告する。
- 問題が起きた場合は、原因追跡と修復がしやすい変更を心がける。

## レビュー・設計・テスト計画の記録

- レビュー・設計・テスト計画は、対象プロジェクトの`DEVELOPMENT.md`へMarkdownで残す。
- レビューには、対象・根拠となる入力と結果・優先度を記載する。
- 設計には、責務・公開API・不変条件・判断理由を短く記載する。
- テスト計画はチェックリストにし、作業前に作成して進捗に合わせて更新する。
- Red・Green・Refactorの結果と検証コマンドを記録する。
- 最初から成功したテストや環境エラーを、Red確認済みとして記録しない。
- 既存の記録を更新し、未実装事項と実装済みの内容を区別する。

## 製造
- ゴールとしてソースコードが動くことよりもソースコードの可読性と保守性が高いことを重視してください。
- ドメイン層では検証や変換を行わず、インフラ層で検証や変換を済ませてください。例えば標準入力を受け取ってチームを設定する構造体があるとして、ドメインモデルにあった形に検証して変換するのが適切です。
- 煩雑になりそうな箇所は擬似コードとして実装してもいいです。
- 悪い例、良い例に関して以下に提示します。
### 悪い例
エラーハンドリングや速くプログラム作成することを重視しすぎてコードが肥大化している。
```rust
fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
    self.validate_skittles(skittles);
    let before: HashSet<_> = skittles
        .iter()
        .map(|pin| (pin.id(), pin.number()))
        .collect();
    let updated_skittles = judge(stick, skittles);
    self.validate_skittles(&updated_skittles);
    let after: HashSet<_> = updated_skittles
        .iter()
        .map(|pin| (pin.id(), pin.number()))
        .collect();
    assert_eq!(before, after, "判定後のスキットル集合が変わっています");
    updated_skittles
}

fn next_team(&self, teams: &[Team]) -> Vec<Team> {
    let mut turns = teams.iter().enumerate().filter(|(_, team)| team.turn());
    let current = turns.next().map(|(index, _)| index);
    assert!(turns.next().is_none(), "手番のチームが複数います");
    let remaining = teams
        .iter()
        .filter(|team| team.score_board().state() != ScoreState::Disqualified)
        .count();
    let next = if teams.is_empty()
        || self.judge_game(teams).is_some()
        || (teams.len() >= 2 && remaining == 1)
    {
        None
    } else {
        let start = current.map_or(0, |index| (index + 1) % teams.len());
        (0..teams.len())
            .map(|offset| (start + offset) % teams.len())
            .find(|&index| teams[index].score_board().state() != ScoreState::Disqualified)
    };
    teams
        .iter()
        .enumerate()
        .map(|(index, team)| team.with_turn(next == Some(index)))
        .collect()
}
```
### 良い例
最小限のプログラムを作成して、動くことより、可読性と保守性が高いことを目指している。
```rust
fn hit(&self, stick: &Stick, skittles: &[Skittle], judge: JudgeStrategy<'_>) -> Vec<Skittle> {
    let updated_skittles = judge(stick, skittles);
    if updated_skittles.iter().counts().any(|(k, v)| v > 1) {
        return updated_skittles;
    }
    return updated_skittles;
}

fn next_team(&self, teams: &[Team]) -> Option<&Team> {
    let turns = teams.iter().filter(|team| team.turn());
    let current = turns.next();
    current
}
```

## 検証コマンド

- Rust：`cd rust_molkky && cargo test`
- 変更対象に応じて検証し、実行していない確認項目は明示する。
