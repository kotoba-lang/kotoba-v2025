# Kotobasos Holochain Implementation

Holochain版のkotobasos実装。エージェント中心の分散型プラットフォームで、Kernel + Actor + Mediator + Evolution Engineパターンを実現します。

## 概要

このクレートは、既存の`kotoba-os`の機能をHolochainのエージェント中心アーキテクチャに適応させた実装です。

### 主な特徴

- **エージェント中心**: 各エージェントが独自のソースチェーンを持ち、DHTで協調
- **DHT直接操作**: Holochain DHTを直接使用してMerkle DAGを構築
- **自己進化**: Evolution Engineによる自動最適化
- **分散実行**: エージェント間でのプロセス協調実行

## アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                    Holochain Zome                           │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │   Kernel     │   │   Mediator   │   │    Actor    │      │
│ │ (Orchestrate)│   │ (Select Actor)│   │ (Perform)   │      │
│ └──────┬───────┘   └──────┬───────┘   └──────┬──────┘      │
└────────┼───────────────────┼──────────────────┼─────────────┘
         │ DHT              │ DHT              │ DHT
         ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│              Holochain DHT (Merkle DAG)                     │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │ Story        │   │ Provenance  │   │ Evolution   │      │
│ │ Process      │   │ MerkleNode  │   │ Actor       │      │
│ └──────────────┘   └──────────────┘   └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## コンポーネント

### 1. DHT操作 (`dht.rs`)

Holochain DHTを直接操作してMerkle DAGを構築する機能：

- `store_jsonld_entry()`: JSON-LDデータをDHTエントリとして保存（CIDインデックスも自動作成）
- `get_jsonld_entry()`: DHTからJSON-LDデータを取得
- `query_dht()`: DHTクエリ実行（エントリタイプとフィルタ条件に基づく）
- `resolve_cid()`: CID（Content ID）からDHTエントリを解決
- `build_merkle_dag()`: Merkle DAG構造をDHT上に構築

### 2. Merkle DAG (`merkle.rs`)

DHT上でのMerkle DAG実装：

- `MerkleNode`: DHTエントリとしてのMerkleノード
- `link_nodes()`: Merkleノード間のリンクを作成・更新
- `verify_merkle_path()`: Merkleパスの検証
- `traverse_dag()`: DAGの深さ優先探索

### 3. Kernel (`kernel.rs`)

Holochain対応Kernel実装：

- `HolochainKernel`: エージェント中心モデルに適応したKernel
- `run_process()`: StoryとProcessをDHTから取得して実行
- `evolve()`: Evolution Engine実行
- `get_evolution_history()`: 進化履歴を取得

### 4. Mediator (`mediator.rs`)

DHTベースのアクター検索と選択：

- `HolochainMediator`: DHTを利用したアクター検索
- `select_actor()`: DHT上でアクターを検索して選択

### 5. Provenance (`provenance.rs`)

DHT上での実行履歴記録：

- `HolochainProvenance`: PROV-O形式のJSON-LDをDHTエントリとして保存
- `record()`: 実行履歴を記録

### 6. Evolution Engine (`evolution.rs`)

DHT上でEvolution Engineを実行：

- `HolochainEvolutionEngine`: OWL推論による最適化パターン発見
- `evolve()`: Provenanceデータを取得してOWL推論を実行
- `extract_patterns()`: SPARQLクエリでパターンを抽出
- `analyze_performance_metrics()`: パフォーマンスメトリクスを分析

### 7. Zome関数 (`zome.rs`)

Holochainのzome関数としてKernel機能を公開：

- `create_story()`: 新しいストーリーを作成
- `run_process()`: プロセスを実行
- `register_actor()`: アクターを登録
- `get_provenance()`: 実行履歴を取得
- `evolve()`: Evolution Engine実行
- `get_evolution_history()`: 進化履歴を取得

## 使用方法

### Zome関数の呼び出し例

```rust
// Storyを作成
let story_json = json!({
    "@context": "...",
    "@graph": [...]
});
let story_hash = create_story(story_json).await?;

// Actorを登録
let actor_input = RegisterActorInput {
    actor_id: "actor:1".to_string(),
    capability: "kotoba:capability/execution".to_string(),
    metadata: None,
};
let actor_hash = register_actor(actor_input).await?;

// プロセスを実行
let process_input = RunProcessInput {
    process_id: "process:1".to_string(),
    story_id: "story:1".to_string(),
};
let result = run_process(process_input).await?;

// Evolution Engineを実行
let evolve_input = EvolveInput {
    provenance_id: result.provenance_id,
    evolution_type: EvolutionType::Hybrid,
};
let evolution = evolve(evolve_input).await?;
```

## 実装の詳細

### CIDインデックス

各エントリは自動的にCIDインデックスエントリを作成し、CIDからEntryHashへのマッピングを保持します。これにより、CIDベースの検索が可能になります。

### DHTクエリ

エントリタイプとフィルタ条件に基づいてDHTをクエリします。現在の実装では、エントリタイプのハッシュからリンクを辿る方式を使用しています。

### Merkle DAG

Merkle DAG構造はDHT上に直接構築されます。各ノードは`MerkleNodeEntry`として保存され、親子関係はリンクとCIDで管理されます。

### Evolution Engine

OWL推論エンジン（`kotoba-owl-reasoner`）を使用して、Provenanceデータから最適化パターンを発見します。SPARQLクエリで共起パターンやパフォーマンスパターンを抽出します。

## 要件

- Holochain HDK v0.4.x
- Rust 1.70+
- `kotoba-os` (with `reasoning` feature)
- `kotoba-owl-reasoner`
- `kotoba-jsonld`

## DNA定義

`dna/kotobasos.dna.yaml`で以下のエントリタイプを定義：

- `story`: ストーリー（プロセスネットワーク）
- `process`: プロセス定義
- `provenance`: 実行履歴
- `evolution`: 進化提案
- `merkle_node`: Merkle DAGノード
- `actor`: Actor定義

## テスト

### 単体テスト

```bash
cargo test --lib
```

各モジュールの単体テストを実行します。

### 統合テスト

```bash
cargo test --test integration_tests
```

Holochainテスト環境での動作確認を行います（`#[ignore]`が付いているテストは手動で実行）。

### エージェント間通信テスト

```bash
cargo test --test agent_communication_tests
```

複数のエージェント間での通信と協調動作をテストします。

## パフォーマンス最適化

### キャッシュ機能

- **CIDインデックスキャッシュ**: CIDからEntryHashへのマッピングをキャッシュ（TTL: 5分）
- **DHTクエリキャッシュ**: クエリ結果をキャッシュ（TTL: 1分）

キャッシュマネージャーを使用することで、DHTクエリのパフォーマンスを向上させます。

```rust
use kotobas_tamaki_holochain::dht::cache::CacheManager;

let cache_manager = CacheManager::default();
// CIDキャッシュを使用
let entry_hash = cache_manager.cid_index().get("cid:test").await;
```

### リトライロジック

DHT操作は自動的にリトライされます：

- 最大リトライ回数: 3回
- 指数バックオフ: 初期1秒、最大30秒
- リトライ可能なエラー: DHTエラー、HDKエラー

```rust
use kotobas_tamaki_holochain::error::RetryExecutor;

let retry_executor = RetryExecutor::default();
let result = retry_executor.execute(|| async {
    // DHT操作
    store_jsonld_entry("Story", &data).await
}).await?;
```

### エラーエスカレーション

エラーは自動的にエスカレートされます：

- **ログ**: すべてのエラーがログに記録
- **メトリクス**: エラーカウントがメトリクスに記録（実装予定）
- **通知**: 重要なエラーが通知チャネルに送信（実装予定）

```rust
use kotobas_tamaki_holochain::error::ErrorEscalator;

let escalator = ErrorEscalator::default();
escalator.escalate(&error);
```

## ライセンス

Apache-2.0

