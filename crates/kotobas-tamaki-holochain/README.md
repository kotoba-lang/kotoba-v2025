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

## 使用方法

### Zome関数

- `create_story`: 新しいストーリー（プロセスネットワーク）を作成
- `run_process`: プロセスを実行
- `register_actor`: アクターを登録
- `get_provenance`: 実行履歴を取得
- `evolve`: Evolution Engine実行
- `get_evolution_history`: 進化履歴を取得

## 要件

- Holochain HDK v0.4.x
- Rust 1.70+

## ライセンス

Apache-2.0

