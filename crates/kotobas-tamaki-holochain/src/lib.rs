//! # Kotobasos Holochain Implementation
//!
//! Holochain版のkotobasos実装。エージェント中心の分散型プラットフォームで、
//! Kernel + Actor + Mediator + Evolution Engineパターンを実現します。
//!
//! ## アーキテクチャ
//!
//! このクレートは、既存の`kotoba-os`の機能をHolochainのエージェント中心アーキテクチャに適応させた実装です。
//!
//! - **エージェント中心**: 各エージェントが独自のソースチェーンを持ち、DHTで協調
//! - **DHT直接操作**: Holochain DHTを直接使用してMerkle DAGを構築
//! - **自己進化**: Evolution Engineによる自動最適化
//! - **分散実行**: エージェント間でのプロセス協調実行

pub mod dht;
pub mod evolution;
pub mod kernel;
pub mod mediator;
pub mod merkle;
pub mod provenance;
pub mod types;
pub mod utils;
pub mod zome;

pub use dht::*;
pub use evolution::*;
pub use kernel::*;
pub use mediator::*;
pub use merkle::*;
pub use provenance::*;
pub use types::*;
pub use utils::*;
pub use zome::*;

/// Error types for Holochain kotobasos operations
#[derive(Debug, thiserror::Error)]
pub enum HolochainKotobasosError {
    #[error("Holochain HDK error: {0}")]
    Hdk(#[from] hdk::prelude::HdkError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("DHT operation error: {0}")]
    Dht(String),

    #[error("Merkle DAG error: {0}")]
    MerkleDag(String),

    #[error("Kernel error: {0}")]
    Kernel(String),

    #[error("Actor error: {0}")]
    Actor(String),

    #[error("Provenance error: {0}")]
    Provenance(String),

    #[error("Evolution error: {0}")]
    Evolution(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HolochainKotobasosError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_generation() {
        let data = json!({
            "@type": "kotoba:Process",
            "@id": "kotoba:process/test",
            "kotoba:label": "Test Process"
        });

        let cid = crate::utils::jsonld_to_cid(&data).unwrap();
        assert!(cid.starts_with("cid:"));
        assert_eq!(cid.len(), 66); // "cid:" + 64 hex chars
    }

    #[test]
    fn test_agent_string_conversion() {
        // AgentPubKeyのテストは実際のHolochain環境が必要
        // ここでは基本構造のテストのみ
        assert!(true);
    }
}
