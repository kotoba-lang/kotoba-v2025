# 次のステップ

## 現在の状態

### ✅ 完了済み
- ワークスペース依存関係の修正
- Holochain CLIがNix経由で利用可能（`holochain_cli 0.5.6`）
- DNAパッケージングツールが利用可能
- 統合テストとエージェント間通信テストの準備完了
- すべてのスクリプトとドキュメントの準備完了

### ⏳ 次のステップ

#### 1. WASMファイルのビルド

`indexmap`バージョン競合の問題を解決するか、クレート単独でビルドを試行：

```bash
# ビルド状態を確認
./scripts/check-wasm-build.sh

# ビルドを試行（ワークスペース問題がある場合は失敗する可能性）
./scripts/build-wasm.sh
```

**問題**: ワークスペース全体のビルドで`indexmap`のバージョン競合が発生します。
これは`cargo-tarpaulin`（dev-dependency）と`kotoba-owl-reasoner`の間の問題です。

**対処法**:
- `kotobas-tamaki-holochain`を単独でビルドする
- または、`indexmap`のバージョンを統一する

#### 2. DNAパッケージの生成

WASMファイルがビルドされたら、DNAパッケージを生成：

```bash
cd crates/kotobas-tamaki-holochain

# Nix経由でHolochain CLIを使用
nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- dna pack dna/

# または、ラッパースクリプトを使用
../scripts/use-holochain-cli.sh dna pack dna/
```

#### 3. 統合テストの実行

WASMファイルとDNAパッケージが準備できたら：

```bash
# 統合テストを実行
./scripts/run-integration-tests.sh

# または、すべてのテストを実行
./scripts/run-all-tests.sh
```

#### 4. エージェント間通信テストの実行

Holochain Sandbox環境をセットアップ：

```bash
# Sandboxの生成
nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- sandbox generate

# Sandboxの実行
nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- sandbox run
```

その後、エージェント間通信テストを実行：

```bash
cargo test --test agent_communication_tests -- --ignored
```

## 便利なコマンド

### Holochain CLIの使用

```bash
# ラッパースクリプトを使用（推奨）
./scripts/use-holochain-cli.sh --version
./scripts/use-holochain-cli.sh dna pack dna/

# または、直接Nixコマンドを使用
nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- <command>
```

### 環境確認

```bash
# Holochain環境の確認
./scripts/setup-holochain-env.sh

# WASMビルド状態の確認
./scripts/check-wasm-build.sh

# 全体的な検証
./scripts/verify-holochain-local.sh
```

## トラブルシューティング

### indexmapバージョン競合

**問題**: ワークスペース全体のビルドで`indexmap`のバージョン競合が発生

**解決方法**:
1. `kotobas-tamaki-holochain`を単独でビルドする
2. `cargo-tarpaulin`のバージョンを更新する
3. `indexmap`のバージョンを統一する

### Holochain CLIが見つからない

**解決方法**:
- Nix経由で使用: `nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- --version`
- ラッパースクリプトを使用: `./scripts/use-holochain-cli.sh --version`

### DNAパッケージの生成に失敗する

**確認事項**:
- WASMファイルが存在するか
- DNAファイルの構文が正しいか
- Holochain CLIが正しく動作するか

## 参考リンク

- Holochain公式ドキュメント: https://developer.holochain.org/
- Holochainインストールガイド: https://developer.holochain.org/docs/install/
- Holochainテストガイド: https://developer.holochain.org/docs/testing/

