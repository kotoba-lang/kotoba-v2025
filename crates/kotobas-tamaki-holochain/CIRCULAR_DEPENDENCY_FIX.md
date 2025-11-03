# 循環依存解決レポート

## 実施した修正

### 1. `kotoba-types`から`kotoba-jsonld`依存を削除

**ファイル**: `crates/010-logic/010-kotoba-types/Cargo.toml`
- `kotoba-jsonld`への依存を削除

**ファイル**: `crates/010-logic/010-kotoba-types/src/canonical_json.rs`
- `kotoba_jsonld::parse_jsonld_to_value`の呼び出しを削除
- 常に`serde_json::from_str`を使用するように変更
- JSON-LDの特殊キー（`@context`, `@id`, `@type`）の処理は維持（通常のJSONキーとして扱う）

### 2. `kotoba-owl-reasoner`から`kotoba-jsonld`依存を削除

**ファイル**: `crates/010-logic/022-kotoba-owl-reasoner/Cargo.toml`
- `kotoba-jsonld`への依存を削除
- `kotoba-owl-reasoner`は`serde_json::Value`を直接使用しているため、`kotoba-jsonld`は不要

## 解決された循環依存

### 以前の循環依存パス
1. `kotoba-types` → `kotoba-jsonld` ✅ 解決
2. `kotoba-jsonld` → `kotoba-cid`
3. `kotoba-cid` → `kotoba-graph-core`
4. `kotoba-graph-core` → `kotoba-codebase`
5. `kotoba-codebase` → `kotoba-types`

### 別の循環依存パス（解決済み）
1. `kotoba-jsonld` → `kotoba-owl-reasoner` (optional) ✅ 解決
2. `kotoba-owl-reasoner` → `kotoba-jsonld` ✅ 解決

## 現在の状態

### ✅ 解決済み
- 循環依存エラーは解消されました
- `kotoba-types`と`kotoba-owl-reasoner`の依存関係を修正

### ⚠️ 残っている問題
- `kotoba-graph-core`にコンパイルエラーがあります（循環依存とは別の問題）
- これにより`kotobas-tamaki-holochain`のビルドがブロックされています

## 次のステップ

1. `kotoba-graph-core`のコンパイルエラーを修正
2. その後、WASMビルドを再試行

## 影響範囲

### 影響を受ける機能
- `kotoba-types`の`canonical_json`モジュール: JSON-LDパース機能が削除され、通常のJSONパースのみに変更
- `kotoba-owl-reasoner`: 変更なし（`serde_json::Value`を直接使用していたため）

### 互換性
- JSON-LDパースが必要な場合は、`kotoba-jsonld`クレートを直接使用する必要があります
- `canonical_json.rs`のテストは通常のJSONに対して動作します

