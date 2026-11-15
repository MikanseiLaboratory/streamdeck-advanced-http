# Advanced HTTP Client（Stream Deckプラグイン）

Stream Deckのキー／ダイヤルから任意のHTTPリクエストを送信するプラグインです。

最新版は [GitHub Releases](https://github.com/MikanseiLaboratory/streamdeck-advanced-http/releases/latest) からダウンロードしてください。

## 対応環境

- Stream Deck 6.9以降
- Windows 10以降 / macOS 10.15以降
- キー（Keypad）とStream Deck +のダイヤル（Encoder）

## アクション

HTTPメソッドごとにアクションを分割しています。

- キー用: **HTTP GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS / QUERY**
- ダイヤル用: **HTTP GET (Dial)** など、各メソッドの `(Dial)` アクション

キーは押下時（Press）と解放時（Release）、ダイヤルは左右の回転・押し込み・解放・タッチに別々のURLを設定できます。BODYはGET / HEAD / OPTIONS以外で表示されます。

既存の **HTTP REQUEST** / **HTTP REQUEST (DIAL)** アクションは「Any Method」として残してあります。v1で作成したキーと設定はそのまま動作します（`method` / `url_press` / `url_left` などの設定キーは変更していません）。

### 認証

- Basic AUTH ID と Password が両方入っている場合はBasic認証を使います。
- Basic認証が空で Authorization Header が入っている場合は、その値をそのままヘッダーに設定します。
- リクエストが送れなかった場合（URL・メソッド・ヘッダー不正、30秒のタイムアウトなど）は、設定に応じてWARNINGを表示します。HTTPステータスコードはログに記録します。
- 4xx / 5xx でもレスポンスが返ればv1と同じくOK扱いです。

## ビルド

```sh
cargo test --workspace
cargo run -p advanced-http-plugin --bin typegen
cd pi && npm ci && npm run build
```

`PACK=1 ./publish.sh` で `.streamDeckPlugin` を作成できます。Windowsでは `publish.ps1` を使います。

タグ `v2.0.0.0`（alpha/beta/rcは `-alpha` など）をpushすると、GitHub ActionsがWindows/macOS向けにビルドしてGitHub Releaseを作成します。タグの4桁バージョンは `manifest.json` の `Version` と一致している必要があります。

## ライセンス

[MIT](LICENSE)
