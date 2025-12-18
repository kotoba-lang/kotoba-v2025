# ワークスペース依存関係修正レポート

## 修正日時
2024年11月3日

## 修正内容

### 1. kotoba-core依存の削除
以下のクレートから存在しない`kotoba-core`への依存を削除またはコメントアウトしました：

- `crates/030-storage/031-kotoba-storage/Cargo.toml`
- `crates/030-storage/036-kotoba-memory/Cargo.toml`

**理由**: ソースコードで`kotoba-core`を使用していないため、不要な依存関係でした。

### 2. パスの修正
以下の依存関係のパスを`010-core`から`010-logic`に修正しました：

- `kotoba-logic`: `../../010-core/009-kotoba-logic` → `../../010-logic/009-kotoba-logic`
- `kotoba-types`: `../../010-core/010-kotoba-types` → `../../010-logic/010-kotoba-types`
- `kotoba-ir`: `../../010-core/011-kotoba-ir` → `../../010-logic/011-kotoba-ir`
- `kotoba-rewrite-kernel`: `../../010-core/012-kotoba-rewrite-kernel` → `../../010-logic/012-kotoba-rewrite-kernel`
- `kotoba-cid`: `../../010-core/013-kotoba-cid` → `../../010-logic/013-kotoba-cid`
- `kotoba-schema`: `../../010-core/014-kotoba-schema` → `../../010-logic/014-kotoba-schema`
- `kotoba-auth`: `../../010-core/015-kotoba-auth` → `../../010-logic/015-kotoba-auth`
- `kotoba-graph-core`: `../../010-core/016-kotoba-graph-core` → `../../010-logic/016-kotoba-graph-core`
- `kotoba-txlog`: `../../010-core/017-kotoba-txlog` → `../../010-logic/017-kotoba-txlog`
- `kotoba-api`: `../../010-core/018-kotoba-api` → `../../010-logic/018-kotoba-api`

### 3. kotoba-codebase依存のコメントアウト
存在しない`kotoba-codebase`への依存をコメントアウトしました：

- 複数の`Cargo.toml`ファイル

### 4. kotoba-osのfeatures修正
`kotobas-tamaki-holochain`の`kotoba-os`依存から`features = ["reasoning"]`を削除しました：

- `crates/kotobas-tamaki-holochain/Cargo.toml`

**理由**: `kotoba-os`に`reasoning`フィーチャーが存在しないため。

## 既知の問題

### indexmapバージョン競合
`cargo-tarpaulin`（dev-dependency）と`kotoba-owl-reasoner`の間で`indexmap`のバージョン競合が発生しています：

```
error: failed to select a version for `indexmap`.
    ... required by package `cargo-tarpaulin v0.29.0`
    ... which satisfies dependency `cargo-tarpaulin = "^0.29"` of package `kotoba-main v0.1.22`
versions that meet the requirements `~1.8` are: 1.8.2, 1.8.1, 1.8.0

all possible versions conflict with previously selected packages.
  previously selected package `indexmap v1.9.3`
    ... which satisfies dependency `indexmap = "^1.9.3"` of package `rdf-types v0.15.2`
```

**影響**: これは`kotoba-main`パッケージのdev-dependencyの問題で、`kotobas-tamaki-holochain`のビルドには直接影響しません。

**対処法**: 
- `kotobas-tamaki-holochain`を単独でビルドする場合は問題ありません
- ワークスペース全体をビルドする場合は、`cargo-tarpaulin`のバージョンを更新するか、`indexmap`のバージョンを統一する必要があります

## 次のステップ

1. `kotobas-tamaki-holochain`を単独でビルドして動作確認
2. WASMビルドの実行
3. 単体テストの実行
4. Holochain環境のセットアップ（統合テスト用）

