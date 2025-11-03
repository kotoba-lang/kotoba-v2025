# Kotoba JSON-LD

JSON-LD utilities for Kotoba graph processing system.

## 概要

Kotoba JSON-LD は、Kotoba のグラフ処理における JSON-LD 形式のデータを扱うためのクレートです。JSON-LD パーサー/シリアライザー、@context 解決機能、JSON Schema ↔ JSON-LD 変換ユーティリティなどを提供します。

## 主な機能

- **JSON-LD パーサー/シリアライザー**: JSON-LD ドキュメントの解析と生成
- **@context 解決**: GitHub URL からの context 取得と解決
- **JSON Schema ↔ JSON-LD 変換**: JSON Schema と JSON-LD の相互変換
- **Context キャッシュ**: パフォーマンス向上のための context キャッシュ機能

## 使用例

```rust
use kotoba_jsonld::{parse_jsonld, serialize_jsonld, ContextResolver};

// JSON-LD ドキュメントをパース
let jsonld_str = r#"
{
    "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
    "@type": "kotoba:User",
    "name": "Alice"
}
"#;

let doc = parse_jsonld(jsonld_str)?;

// Context を解決
let resolver = ContextResolver::new();
let expanded = expand_jsonld(&doc, &resolver).await?;

// JSON-LD をシリアライズ
let serialized = serialize_jsonld(&doc)?;
```

## Context 解決

GitHub URL からの context 解決をサポートします：

```rust
let resolver = ContextResolver::new();
let context = resolver.resolve(
    "https://github.com/com-junkawasaki/kotoba/blob/main/schemas/kotoba-context.jsonld"
).await?;
```

GitHub blob URL は自動的に raw URL に変換されます。

## JSON Schema 変換

JSON Schema を JSON-LD に変換：

```rust
use kotoba_jsonld::json_schema_to_jsonld;

let schema = json!({
    "$id": "https://example.org/schema",
    "title": "User",
    "properties": {
        "name": {"type": "string"}
    }
});

let jsonld = json_schema_to_jsonld(&schema, None)?;
```

## 依存関係

- `kotoba-types`: 基本型定義
- `kotoba-cid`: CID システム
- `serde_json`: JSON 処理
- `reqwest`: HTTP クライアント（context 解決用）

## アーキテクチャ

- **JsonLdDocument**: JSON-LD ドキュメントの構造化表現
- **JsonLdContext**: @context フィールドの表現（文字列、オブジェクト、配列）
- **ContextResolver**: URL からの context 解決機能
- **変換ユーティリティ**: JSON Schema ↔ JSON-LD 変換

