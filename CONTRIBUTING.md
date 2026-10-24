# コントリビュートについて

IssueとPull Requestを歓迎します。

## 開発環境

- Windows(ICMP、サービス、Named Pipe、DPAPIはWindows専用)
- Rust(stable)

## 提出前の確認

CIと同じ内容です。すべて通してからPull Requestを出してください。

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 方針

- 変更は目的ごとに小さく分け、挙動を変える場合はテストを追加してください
- 設定項目、CLI、データモデルを変えた場合は`README.md`も更新してください
- Windowsサービスの動作(`install` / `start`など)は管理者権限が必要でCIでは検証できません。関係する変更では、手元での確認結果をPull Requestに書いてください

## バグ報告

次の情報があると調査しやすくなります。

- Windowsのバージョン
- `monitord --version`と、サービス実行かコンソール実行か
- 設定ファイル(Webhook URLは必ず伏せる)
- `<data_dir>\logs\`のログ

## セキュリティ上の問題

Webhook URLの漏えいやIPCの権限など、セキュリティに関わる問題は公開Issueではなく、GitHubのSecurity Advisory(Report a vulnerability)から非公開で報告してください。

## ライセンス

コントリビュートされた内容は、本プロジェクトと同じ`MIT OR Apache-2.0`のデュアルライセンスで提供されるものとします。
