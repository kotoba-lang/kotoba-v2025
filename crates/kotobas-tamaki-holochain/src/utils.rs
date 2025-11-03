//! ユーティリティ関数

use crate::Result;
use hdk::prelude::*;
use serde_json::Value;

/// JSON-LDデータをCID（Content ID）に変換
pub fn jsonld_to_cid(data: &Value) -> Result<String> {
    use sha2::{Digest, Sha256};
    
    let json_bytes = serde_json::to_vec(data)?;
    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    let hash = hasher.finalize();
    
    Ok(format!("cid:{}", hex::encode(hash)))
}

/// CIDからJSON-LDデータを取得（DHTから解決）
/// 注意: この関数は実際の実装ではEntryHashの解決方法を変更する必要がある
pub async fn cid_to_jsonld(_cid: &str) -> Result<Value> {
    // TODO: CIDからEntryHashへの変換を実装
    // 現在はプレースホルダー
    Err(crate::HolochainKotobasosError::Dht("Not implemented yet".to_string()))
}

/// エージェントIDを文字列に変換
pub fn agent_to_string(agent: &AgentPubKey) -> String {
    format!("agent:{}", agent)
}

/// 文字列からエージェントIDを解析
/// 注意: AgentPubKeyの実際のシリアライゼーション形式に依存
pub fn string_to_agent(_s: &str) -> Result<AgentPubKey> {
    // TODO: 実際のAgentPubKeyの解析を実装
    // 現在はプレースホルダー
    Err(crate::HolochainKotobasosError::Other(anyhow::anyhow!("Not implemented yet")))
}

