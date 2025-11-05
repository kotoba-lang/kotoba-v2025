//! DHT上でのProvenance実装
//!
//! 実行履歴をDHT上に記録します。

use crate::dht::store_jsonld_entry;
use crate::types::{ActorEntry, ProvenanceEntry};
use crate::Result;
use hdk::prelude::*;
use serde_json::{json, Value};

/// Holochain Provenance
pub struct HolochainProvenance;

impl HolochainProvenance {
    /// 新しいProvenanceサービスを作成
    pub fn new() -> Self {
        Self
    }

    /// Provenanceを記録
    pub async fn record(
        &self,
        process_id: &str,
        actor: &ActorEntry,
        result: &Value,
    ) -> Result<String> {
        let agent = agent_info()?.agent_latest_pubkey();

        let provenance_entry = ProvenanceEntry {
            id: crate::utils::jsonld_to_cid(result)?,
            provenance: json!({
                "@type": "prov:Activity",
                "prov:wasGeneratedBy": process_id,
                "prov:wasAssociatedWith": actor.id,
                "prov:endedAtTime": chrono::Utc::now().to_rfc3339(),
            }),
            process_id: process_id.to_string(),
            executed_at: chrono::Utc::now().timestamp(),
            executor: agent,
        };

        let entry_value = serde_json::to_value(&provenance_entry)?;
        let _entry_hash = store_jsonld_entry("Provenance", &entry_value).await?;

        Ok(provenance_entry.id)
    }
}

