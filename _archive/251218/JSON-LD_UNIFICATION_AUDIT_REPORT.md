# JSON-LD統一状況の調査・分析・評価レポート

## エグゼクティブサマリー

プロジェクト全体でJSON-LDへの統一状況を調査した結果、**主要なAPI/データ処理レイヤーは既にJSON-LD対応済み**ですが、**テストコード、TypeScriptクライアント、一部のユーティリティ関数**で通常のJSONが使用されている箇所が複数見つかりました。

## 調査方法

1. `serde_json::from_str`, `serde_json::from_slice`などの直接パース箇所を検索
2. `serde_json::json!`マクロの使用箇所を検索
3. HTTPリクエスト/レスポンスでの`application/json`の使用箇所を検索
4. 残存する`.json`ファイルの確認

## 調査結果

### カテゴリA: 即座にJSON-LD対応が必要（高優先度）

#### A1. TypeScriptクライアント（packages/kotobajs）
**ファイル:**
- `packages/kotobajs/src/workflow.ts` (line 19)
- `packages/kotobajs/src/client.ts` (line 23)

**問題:**
- `Content-Type: application/json`をハードコード
- APIサーバーは既に`application/ld+json`を要求

**影響:**
- TypeScriptクライアントがAPIと通信できない

**対応方針:**
- `Content-Type`を`application/ld+json`に変更
- リクエスト/レスポンスをJSON-LD形式に変換

**推定作業量:** 2時間

#### A2. JSON正規化ユーティリティ（canonical_json.rs）
**ファイル:**
- `crates/010-logic/010-kotoba-types/src/canonical_json.rs`

**問題:**
- JSON正規化処理が通常のJSONを前提としている
- JSON-LDの`@context`や`kotoba:`プレフィックスを考慮していない

**影響:**
- JSON-LDデータの正規化が正しく動作しない可能性

**対応方針:**
- JSON-LD対応の正規化処理を実装
- `@context`と`@id`を優先的に処理
- `kotoba:`プレフィックス付きキーのソート順を考慮

**推定作業量:** 4時間

#### A3. Jsonnet評価結果のパース（kotoba-jsonnet）
**ファイル:**
- `crates/020-language/023-kotoba-jsonnet/src/evaluator.rs` (line 58)
- `crates/020-language/023-kotoba-jsonnet/src/stdlib.rs` (line 1408)

**問題:**
- Jsonnetの評価結果を通常のJSONとしてパース
- JSON-LD形式への変換が行われていない

**影響:**
- Jsonnetから生成されるデータがJSON-LD形式にならない

**対応方針:**
- Jsonnet評価結果をJSON-LD形式に変換する処理を追加
- `@context`を自動付与

**推定作業量:** 3時間

#### A4. 設定ファイル（config.kotoba.json）
**ファイル:**
- `examples/http_server/config.kotoba.json` (line 42)

**問題:**
- `content_type: "application/json"`が設定されている

**影響:**
- HTTPサーバーの設定がJSON-LDに対応していない

**対応方針:**
- 設定ファイルをJSON-LD形式に変換し、`application/ld+json`に変更

**推定作業量:** 30分

#### A5. コード生成（kotoba2tsx generator）
**ファイル:**
- `crates/020-language/028-kotoba2tsx/src/generator.rs` (line 732)

**問題:**
- 生成されるTypeScriptコードで`Content-Type: application/json`がハードコード

**影響:**
- 生成されたコードがJSON-LD APIと互換性がない

**対応方針:**
- コード生成テンプレートを`application/ld+json`に変更

**推定作業量:** 1時間

#### A6. AI Models統合（kotoba-kotobas）
**ファイル:**
- `crates/020-language/023-kotoba-kotobas/src/ai_models.rs` (line 100)
- `crates/020-language/023-kotoba-kotobas/src/config.rs` (line 1088)

**問題:**
- 外部AI APIへのリクエストで`application/json`を使用

**影響:**
- 外部APIとの統合がJSON-LDに対応していない

**対応方針:**
- 外部APIは通常のJSONを受け付けるため、リクエスト時のみJSON形式に変換
- 内部データはJSON-LD形式を維持

**推定作業量:** 2時間

### カテゴリB: 段階的に対応可能（中優先度）

#### B1. テストコードのJSONデータ生成
**ファイル（多数）:**
- `tests/integration/src/query_engine_tests.rs` (lines 42-66)
- `tests/integration/src/graph_rewriting_tests.rs` (lines 70-80)
- `tests/integration/src/performance_tests.rs` (line 288)
- `tests/integration/src/ocel_graphdb_tests.rs` (line 263)
- `tests/load/src/reporter.rs` (lines 133-159)
- `tests/load/src/metrics.rs` (line 387)

**問題:**
- テストフィクスチャで`serde_json::json!`マクロを使用
- JSON-LD形式になっていない

**影響:**
- テストデータがJSON-LD形式ではないが、テスト自体は動作する

**対応方針:**
- テストヘルパー関数を作成し、JSON-LD形式のデータを生成
- 既存テストを段階的に移行

**推定作業量:** 8時間

#### B2. トポロジー検証テスト
**ファイル:**
- `tests/topology_validation.rs` (lines 37, 287)

**問題:**
- Jsonnetの出力を通常のJSONとしてパース
- JSON-LD形式への変換が行われていない

**影響:**
- テストは動作するが、JSON-LD形式ではない

**対応方針:**
- Jsonnet出力をJSON-LD形式に変換する処理を追加

**推定作業量:** 2時間

#### B3. サンプルコード・ドキュメント
**ファイル:**
- `examples/web/simple-site.kotoba` (lines 172, 281, 292)
- `examples/web/comprehensive-web-app.kotoba` (line 384)
- `examples/components/advanced-handlers.kotoba` (lines 431, 471, 580)
- `examples/docs/README-Built-in-Handlers.md` (line 66)
- `docs/tutorials/getting-started.md` (複数箇所)
- `docs/deployment/README.md` (line 994)
- `docs/_pages/quickstart.md` (line 229)

**問題:**
- ドキュメントやサンプルコードで`application/json`を使用

**影響:**
- ユーザーが古い形式でコードを書く可能性

**対応方針:**
- ドキュメントとサンプルコードを更新
- `application/ld+json`に変更

**推定作業量:** 4時間

### カテゴリC: 対応不要（低優先度・外部依存）

#### C1. 設定ファイル（package.json, tsconfig.json等）
**ファイル:**
- `packages/kotobajs/package.json`
- `packages/kotobajs/tsconfig.json`
- `examples/tauri_react_app/package.json`
- `examples/tauri_react_app/tsconfig.json`
- `examples/vercel/vercel.json`

**理由:**
- 外部ツール（npm, TypeScript, Vercel等）の設定ファイル
- JSON-LD対応は不要

#### C2. テストベンチマーク・ユーティリティ
**ファイル:**
- `test_pure_functional.rs` (line 167)
- `examples/pure_functional_test.rs` (line 159)
- `benches/pure_functional_benchmark.rs` (line 188)

**理由:**
- テスト/ベンチマーク用の一時的なJSON使用
- 本番コードには影響しない

#### C3. 外部ライブラリ統合
**ファイル:**
- `crates/020-language/023-kotoba-jsonnet/src/stdlib.rs` (line 1408)
  - `std.parseIntJSON()`関数でのJSONパース（標準ライブラリ関数）

**理由:**
- 外部ライブラリとの統合インターフェース
- JSON-LD対応は外部ライブラリ側で対応が必要

## 優先度マトリックス

| カテゴリ | ファイル数 | 影響範囲 | 優先度 | 推定作業量 |
|---------|-----------|---------|--------|-----------|
| A1: TypeScriptクライアント | 2 | 高 | 最高 | 2時間 |
| A2: JSON正規化 | 1 | 中 | 高 | 4時間 |
| A3: Jsonnet評価結果 | 2 | 中 | 高 | 3時間 |
| A4: 設定ファイル | 1 | 低 | 中 | 30分 |
| A5: コード生成 | 1 | 中 | 高 | 1時間 |
| A6: AI Models | 2 | 中 | 中 | 2時間 |
| B1: テストコード | 6+ | 低 | 中 | 8時間 |
| B2: トポロジー検証 | 1 | 低 | 低 | 2時間 |
| B3: ドキュメント | 8+ | 低 | 中 | 4時間 |

**合計推定作業量:** 約26.5時間

## 推奨対応順序

### Phase 1: 緊急対応（即座に実行）
1. **A1: TypeScriptクライアント** - APIとの互換性問題
2. **A5: コード生成** - 生成コードの互換性問題

### Phase 2: 重要機能（1週間以内）
3. **A2: JSON正規化** - データ整合性に影響
4. **A3: Jsonnet評価結果** - データ生成パイプラインに影響
5. **A6: AI Models** - 外部統合に影響

### Phase 3: 品質向上（2週間以内）
6. **A4: 設定ファイル** - 設定の統一
7. **B3: ドキュメント** - ユーザー体験の向上

### Phase 4: 完全統一（1ヶ月以内）
8. **B1: テストコード** - テストデータの統一
9. **B2: トポロジー検証** - 検証パイプラインの統一

## リスク評価

### 高リスク
- **TypeScriptクライアント（A1）**: クライアントとサーバー間の通信が失敗する可能性
- **コード生成（A5）**: 生成されたコードが動作しない可能性

### 中リスク
- **JSON正規化（A2）**: データ整合性チェックが正しく動作しない可能性
- **Jsonnet評価結果（A3）**: データ生成パイプラインで形式不整合が発生する可能性

### 低リスク
- **テストコード（B1）**: テストは動作するが、データ形式が統一されていない
- **ドキュメント（B3）**: ユーザーが混乱する可能性

## 結論

主要なAPI/データ処理レイヤーは既にJSON-LD対応済みですが、**TypeScriptクライアントとコード生成**で緊急の対応が必要です。これらを優先的に対応することで、プロジェクト全体のJSON-LD統一が完了します。

**推奨アクション:**
1. Phase 1の項目を即座に実行
2. Phase 2の項目を1週間以内に完了
3. Phase 3以降は段階的に対応

