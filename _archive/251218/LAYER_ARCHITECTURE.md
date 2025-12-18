# Kotoba Layer Architecture

## Overview

Kotoba follows a layered architecture with clear separation of concerns. Each layer has a specific responsibility and depends only on lower layers.

## Layer Structure

### Layer 005: Foundation Layer (Priority: 1)
**責務**: システムの基盤となる基本データ構造とユーティリティ

- `kotoba-types` - 基本型定義
- `kotoba-cid` - Content ID
- `kotoba-schema` - スキーマ定義
- `kotoba-auth` - 認証基盤
- `kotoba-graph-core` - グラフコア
- `kotoba-logic` - 論理基盤

### Layer 010: Logic Layer (Priority: 2)
**責務**: 中間表現（IR）、リライトカーネル、JSON-LD処理

- `kotoba-ir` - IR (Rule/Query/Patch/Strategy/Catalog)
- `kotoba-rewrite-kernel` - DPO Graph Rewriting Kernel
- `kotoba-jsonld` - JSON-LD処理
- `kotoba-codebase` - コードベース管理
- `kotoba-txlog` - トランザクションログ
- `kotoba-api` - API基盤
- `kotoba-phonosemantic` - 音韻意味システム

### Layer 012: VM Layer (Priority: 3)
**責務**: 仮想マシン実行環境

- `kotoba-vm-core` - VM統合
- `kotoba-vm-memory` - メモリ管理
- `kotoba-vm-cpu` - CPU Core
- `kotoba-vm-scheduler` - スケジューラー
- `kotoba-vm-gnn` - GNN最適化
- `kotoba-vm-hardware` - ハードウェア抽象化
- `kotoba-vm-types` - VM型定義

### Layer 014: Reasoner Layer (Priority: 4)
**責務**: 意味推論エンジン

- `kotoba-owl-reasoner` - OWL推論（RDFS/OWL Lite/OWL DL）

### Layer 015: OS Layer (Priority: 5)
**責務**: プロセスネットワークオーケストレーション

- `kotoba-os` - Kernel + Actor + Mediator

### Layer 020: Language Layer (Priority: 6)
**責務**: 言語処理（Parser、Analyzer、Transpiler）

- `kotoba-syntax` - 構文定義
- `kotoba-parser` - パーサー
- `kotoba-analyzer` - 解析器
- `kotoba-jsonnet` - Jsonnet評価器
- `kotoba-kotobas` - KotobaScript
- `kotoba-formatter` - フォーマッター
- `kotoba-linter` - リンター
- `kotoba-lsp` - LSP
- `kotoba-repl` - REPL
- `kotoba2tsx` - TypeScript Transpiler
- `kotobas-wasm` - WASM変換
- `kotoba-language` - 統合API

### Layer 030: Storage Layer (Priority: 7)
**責務**: 永続化層（Port/Adapter）

- `kotoba-storage` - Storage抽象化
- `kotoba-cache` - キャッシュ
- `kotoba-db-cluster` - DBクラスター
- `kotoba-distributed` - 分散ストレージ
- `kotoba-graphdb` - GraphDB
- `kotoba-memory` - メモリストレージ
- `kotoba-storage-redis` - Redis実装
- `kotoba-storage-rocksdb` - RocksDB実装
- `kotoba-storage-fcdb` - FCDB実装

### Layer 040: Runtime Layer (Priority: 8)
**責務**: OS + Storage + Reasoner統合、アプリケーション実行環境

- (将来実装予定)

### Layer 050: Workflow Layer (Priority: 9)
**責務**: ワークフローオーケストレーション

- `kotoba-workflow-core`
- `kotoba-workflow`
- `kotoba-workflow-activities`
- `kotoba-workflow-operator`

### Layer 060: Application Layer (Priority: 10)
**責務**: ビジネスロジック、イベントソーシング、クエリ処理

- `kotoba-event-stream`
- `kotoba-projection-engine`
- `kotoba-rewrite`
- `kotoba-query-engine`
- `kotoba-execution`
- `kotoba-handler`
- `kotoba-routing`
- `kotoba-state-graph`

### Layer 070: Services Layer (Priority: 11)
**責務**: HTTP/GraphQLサーバー、外部統合

- `kotoba-security`
- `kotoba-network`
- `kotoba-schema-registry`
- `kotoba-server-core`
- `kotoba-graph-api`
- `kotoba-server-workflow`
- `kotoba-server`
- `kotoba-monitoring`
- `kotoba-profiler`
- `kotoba-cloud-integrations`

### Layer 080: Deployment Layer (Priority: 12)
**責務**: デプロイメント、スケーリング、ネットワーキング

- `kotoba-deploy-core`
- `kotoba-deploy`
- `kotoba-deploy-scaling`
- `kotoba-deploy-network`
- `kotoba-deploy-git`
- `kotoba-deploy-controller`
- `kotoba-deploy-cli`
- `kotoba-deploy-runtime`
- `kotoba-deploy-hosting`

### Layer 090: Tools Layer (Priority: 13)
**責務**: 開発ツール、CLI、ビルドツール

- `kotoba-config`
- `kotoba-build`
- `kotoba-package-manager`
- `kotoba-runtime`
- `kotoba-docs`
- `kotoba-ssg`
- `kotoba-tester`
- `kotoba-bench`
- `kotoba-backup`
- `kotoba-cli`

## Dependency Rules

1. **Lower layers depend only on Foundation Layer (005)**
2. **Higher layers can depend on multiple lower layers**
3. **No circular dependencies between layers**
4. **Transpiler (kotoba2tsx) is in Language Layer (020)**
5. **OS Layer (015) depends on Logic (010), Reasoner (014), and Storage (030)**
6. **Runtime Layer (040) integrates OS + Storage + Reasoner**

## Layer Relationships

```
Foundation (005) ← Logic (010) ← VM (012)
                          ↓
                    Reasoner (014)
                          ↓
                        OS (015)
                          ↓
                    Language (020)
                          ↓
                    Storage (030)
                          ↓
                    Runtime (040)
                          ↓
                    Workflow (050)
                          ↓
                    Application (060)
                          ↓
                    Services (070)
                          ↓
                    Deployment (080)
                          ↓
                    Tools (090)
```
