//! Holochain固有の型定義
//!
//! Holochainのエージェント中心モデルに適応した型定義

use hdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Holochainエントリタイプの定義
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum KotobasosEntry {
    /// ストーリー（プロセスネットワーク）
    Story(StoryEntry),
    /// プロセス定義
    Process(ProcessEntry),
    /// Provenance（実行履歴）
    Provenance(ProvenanceEntry),
    /// Evolution（進化提案）
    Evolution(EvolutionEntry),
    /// Merkle DAGノード
    MerkleNode(MerkleNodeEntry),
    /// Actor定義
    Actor(ActorEntry),
    /// CIDインデックス
    CidIndex(CidIndexEntry),
}

/// ストーリーエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryEntry {
    /// Story ID（CID）
    pub id: String,
    /// JSON-LD形式のストーリー
    pub story: Value,
    /// 作成時刻
    pub created_at: i64,
    /// 作成者エージェント
    pub author: AgentPubKey,
}

/// プロセスエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessEntry {
    /// Process ID（CID）
    pub id: String,
    /// JSON-LD形式のプロセス
    pub process: Value,
    /// ストーリーIDへの参照
    pub story_id: String,
    /// 作成時刻
    pub created_at: i64,
}

/// Provenanceエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvenanceEntry {
    /// Provenance ID（CID）
    pub id: String,
    /// JSON-LD形式のProvenance（PROV-O）
    pub provenance: Value,
    /// プロセスIDへの参照
    pub process_id: String,
    /// 実行時刻
    pub executed_at: i64,
    /// 実行エージェント
    pub executor: AgentPubKey,
}

/// Evolutionエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvolutionEntry {
    /// Evolution ID（CID）
    pub id: String,
    /// JSON-LD形式の進化提案
    pub evolution: Value,
    /// ベースとなるProvenance ID
    pub provenance_id: String,
    /// 進化タイプ
    pub evolution_type: EvolutionType,
    /// 作成時刻
    pub created_at: i64,
}

/// 進化タイプ
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionType {
    /// パターンベース（OWL推論）
    PatternBased,
    /// パフォーマンスベース（メトリクス駆動）
    PerformanceBased,
    /// ハイブリッド
    Hybrid,
}

/// Merkle DAGノードエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleNodeEntry {
    /// ノードID（CID）
    pub id: String,
    /// データのハッシュ
    pub data_hash: String,
    /// 親ノードへのリンク（CIDs）
    pub parent_links: Vec<String>,
    /// 子ノードへのリンク（CIDs）
    pub child_links: Vec<String>,
    /// メタデータ
    pub metadata: Value,
}

/// Actorエントリ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorEntry {
    /// Actor ID
    pub id: String,
    /// Capability IRI
    pub capability: String,
    /// エージェント
    pub agent: AgentPubKey,
    /// メタデータ
    pub metadata: Value,
    /// 登録時刻
    pub registered_at: i64,
}

/// DHTクエリパラメータ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DhtQuery {
    /// エントリタイプ
    pub entry_type: String,
    /// フィルタ条件
    pub filters: Value,
    /// ページネーション
    pub pagination: Option<Pagination>,
}

/// ページネーション
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pagination {
    /// オフセット
    pub offset: usize,
    /// リミット
    pub limit: usize,
}

/// CIDインデックスエントリ（CIDからEntryHashへのマッピング）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CidIndexEntry {
    /// CID
    pub cid: String,
    /// 対応するEntryHash
    pub entry_hash: EntryHash,
    /// エントリタイプ
    pub entry_type: String,
    /// 作成時刻
    pub created_at: i64,
}

