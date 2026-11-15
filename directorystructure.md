# ディレクトリ構成

```text
crates/advanced-http-client/  HTTPメソッド・URL・ヘッダーの検証とreqwestによる送信
crates/advanced-http-plugin/  Stream Deckアクション、設定、typegen、プラグイン本体
pi/                           Property Inspector (React + Vite + TypeScript)
plugin/dev.flowingspdg.advancedhttp.sdPlugin/
  manifest.json
  images/                     共通の白いアイコン
  propertyinspector/          Viteのビルド出力（Git管理外）
  bin/plugin                  macOSユニバーサルバイナリ（Git管理外）
  bin/plugin.exe              Windows実行ファイル（Git管理外）
.github/workflows/ci.yml      テスト、Windows/macOSビルド、パッケージ検証
.github/workflows/release.yml タグpush時のビルドとGitHub Release作成
publish.sh / publish.ps1      ローカルビルド、パッケージ、インストール
```

- `ActionSettings` は `crates/advanced-http-plugin/src/contracts.rs` が唯一の定義です。`cargo run -p advanced-http-plugin --bin typegen` で `pi/src/generated/` のTypeScriptを更新します。
- アクションUUIDとメソッドの対応は `crates/advanced-http-plugin/src/actions.rs` の `METHOD_ACTIONS` が唯一の定義です。typegenがProperty Inspector用の `actions.ts` を生成します。
- 旧アクションUUID（`.perform` / `.dial`）は後方互換のため残しています。
