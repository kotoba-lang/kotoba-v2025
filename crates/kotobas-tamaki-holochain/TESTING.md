# Kotobasos Holochain テストガイド

## 概要

このドキュメントは、Kotobasos Holochain版のローカル動作検証手順を説明します。

## 前提条件

### 必須
- Rust 1.70以上
- `wasm32-unknown-unknown`ターゲット（自動インストール可能）

### 統合テスト用（オプション）
- Holochain CLI (`holochain` または `hc`)
- Holochainテスト環境

## クイックスタート

### 1. 環境セットアップ

```bash
# WASMターゲットのインストール（自動）
rustup target add wasm32-unknown-unknown

# 検証スクリプトの実行
./scripts/verify-holochain-local.sh
```

### 2. 単体テストの実行

```bash
cd crates/kotobas-tamaki-holochain
cargo test --lib
```

**注意**: 現在、ワークスペースの依存関係の問題により、単体テストが実行できない可能性があります。
この問題は`kotoba-storage`が存在しない`kotoba-core`を参照していることが原因です。

### 3. WASMビルド

```bash
./scripts/build-wasm.sh
```

**注意**: ワークスペースの依存関係の問題により、現在ビルドできない可能性があります。

### 4. 統合テストの実行（Holochain環境が必要）

```bash
# Holochain環境をセットアップ後
./scripts/run-integration-tests.sh
```

## テストファイル構成

### 単体テスト (`tests/unit_tests.rs`)

各モジュールの単体テスト：

- **DHTテスト**: CID生成、CID一貫性
- **型テスト**: Story、Process、Evolution型のシリアライゼーション
- **Merkleテスト**: Merkleノードの作成、メタデータ
- **Mediatorテスト**: Mediatorの作成、Actor追加
- **Provenanceテスト**: Provenanceサービスの作成
- **Evolutionテスト**: Evolution Engineの作成、結果のシリアライゼーション

### 統合テスト (`tests/integration_tests.rs`)

Holochain環境での動作確認：

- Story作成と取得
- プロセス実行フロー
- Provenance記録
- Evolution Engine
- Merkle DAG構築

### エージェント間通信テスト (`tests/agent_communication_tests.rs`)

複数エージェントでの協調動作：

- 単一エージェント実行
- 複数エージェント協調
- Actor発見
- Evolution共有
- DHT複製
- 同時実行

## 検証スクリプト

### `scripts/verify-holochain-local.sh`

全体的な検証を実行：
- WASMターゲットの確認
- 単体テストの実行
- WASMビルドの試行
- DNAファイルの検証
- Holochain環境の確認

### `scripts/build-wasm.sh`

WASMファイルのビルド：
- WASMターゲットの自動インストール
- リリースビルドの実行
- WASMファイルの存在確認

### `scripts/run-integration-tests.sh`

統合テストの実行：
- Holochain環境の確認
- WASMファイルの確認
- DNAファイルの確認
- 統合テストの実行

## 既知の問題と対処法

### ワークスペースの依存関係エラー

**問題**: `kotoba-storage`が存在しない`kotoba-core`を参照

**エラーメッセージ**:
```
failed to read `/Users/junkawasaki/jun784/kotoba/crates/010-core/012-kotoba-core/Cargo.toml`
```

**対処法**:
1. `crates/030-storage/031-kotoba-storage/Cargo.toml`を確認
2. `kotoba-core`への参照を削除または正しいパスに修正
3. または、`kotoba-core`に相当するクレートを特定してパスを修正

### Holochain環境の未セットアップ

**問題**: Holochain CLIがインストールされていない

**対処法**:
- Holochain CLIのインストール: https://developer.holochain.org/docs/install/
- または、統合テストは`#[ignore]`が付いているため、Holochain環境なしでもスキップ可能

## 検証結果の記録

検証結果は以下のファイルに記録されます：

- `/tmp/holochain-unit-tests.log`: 単体テストのログ
- `/tmp/holochain-wasm-build.log`: WASMビルドのログ
- `/tmp/holochain-integration-tests.log`: 統合テストのログ（実行時）
- `VERIFICATION.md`: 検証レポート

## 次のステップ

1. **ワークスペースの依存関係を修正**
   - `kotoba-storage`の依存関係を確認・修正
   - 存在しないクレートへの参照を削除または修正

2. **Holochain環境のセットアップ**
   - Holochain CLIのインストール
   - テスト環境のセットアップ

3. **ビルドとテストの実行**
   - WASMファイルのビルド
   - 単体テストの実行
   - 統合テストの実行（Holochain環境が必要）

4. **検証結果の記録**
   - 各フェーズの結果を記録
   - 問題点と解決方法をドキュメント化

