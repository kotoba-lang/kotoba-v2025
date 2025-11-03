# Holochain環境セットアップガイド

## 概要

Kotobasos Holochain版の統合テストとエージェント間通信テストを実行するには、Holochain環境のセットアップが必要です。

## 前提条件

- Rust 1.70以上
- Nixパッケージマネージャー（推奨）またはCargo

## インストール方法

### 方法1: Nixを使用する場合（推奨）

Nixを使用すると、Holochainの開発環境を簡単にセットアップできます。

```bash
# Holochain開発環境のセットアップ（推奨）
bash <(curl https://holochain.github.io/holochain/setup.sh)

# インストール確認（新しいターミナルセッションで）
hc --version
# または
holochain --version

# または、Holonix Flakeを直接使用する場合
nix run --refresh "github:holochain/holonix?ref=main-0.5#hc-scaffold" -- --version
```

### 方法2: Cargoを使用する場合

```bash
# crates.ioからインストール
cargo install holochain_cli

# または、GitHubリポジトリからビルド
git clone https://github.com/holochain/holochain.git
cd holochain
cargo install --path crates/holochain_cli
```

### 方法3: バイナリをダウンロードする場合

1. https://github.com/holochain/holochain/releases から最新のリリースをダウンロード
2. バイナリを`PATH`に追加

## インストール確認

```bash
# Holochain CLIの確認
which holochain || which hc

# バージョン確認
holochain --version
# または
hc --version
```

## DNAパッケージの生成

WASMファイルがビルドされたら、DNAパッケージを生成できます：

```bash
# WASMファイルのビルド
./scripts/build-wasm.sh

# DNAパッケージの生成
cd crates/kotobas-tamaki-holochain
hc dna pack dna/
# または
holochain dna pack dna/
```

## テスト環境のセットアップ

### Holochain Sandbox（推奨）

```bash
# Sandboxの生成
hc sandbox generate

# Sandboxの実行
hc sandbox run
```

### カスタムテスト環境

Holochainテストフレームワークを使用してカスタムテスト環境を構築できます。

詳細: https://developer.holochain.org/docs/testing/

## 統合テストの実行

Holochain環境がセットアップされたら、統合テストを実行できます：

```bash
# 統合テストの実行
./scripts/run-integration-tests.sh

# または直接実行
cargo test --test integration_tests -- --ignored
```

## トラブルシューティング

### Holochain CLIが見つからない

- `PATH`環境変数を確認
- インストール方法を再確認
- 新しいターミナルセッションを開く

### DNAパッケージの生成に失敗する

- WASMファイルが正しくビルドされているか確認
- `dna/kotobasos.dna.yaml`の構文を確認
- Holochain CLIのバージョンを確認

### テスト環境の起動に失敗する

- Holochain CLIが正しくインストールされているか確認
- 必要なポートが使用可能か確認
- ログを確認してエラーメッセージを特定

## 参考リンク

- Holochain公式ドキュメント: https://developer.holochain.org/
- Holochainインストールガイド: https://developer.holochain.org/docs/install/
- Holochainテストガイド: https://developer.holochain.org/docs/testing/
- Holochain GitHub: https://github.com/holochain/holochain

