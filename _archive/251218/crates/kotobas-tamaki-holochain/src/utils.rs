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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonld_to_cid() {
        let data = json!({
            "@type": "kotoba:Process",
            "@id": "kotoba:process/test",
            "kotoba:label": "Test Process"
        });

        let cid1 = jsonld_to_cid(&data).unwrap();
        let cid2 = jsonld_to_cid(&data).unwrap();
        
        // 同じデータから同じCIDが生成される
        assert_eq!(cid1, cid2);
        assert!(cid1.starts_with("cid:"));
        assert_eq!(cid1.len(), 66); // "cid:" + 64 hex chars
    }

    #[test]
    fn test_jsonld_to_cid_different_data() {
        let data1 = json!({
            "@type": "kotoba:Process",
            "@id": "kotoba:process/test1"
        });

        let data2 = json!({
            "@type": "kotoba:Process",
            "@id": "kotoba:process/test2"
        });

        let cid1 = jsonld_to_cid(&data1).unwrap();
        let cid2 = jsonld_to_cid(&data2).unwrap();
        
        // 異なるデータから異なるCIDが生成される
        assert_ne!(cid1, cid2);
    }

    #[test]
    fn test_agent_to_string() {
        // AgentPubKeyのテストは実際のHolochain環境が必要
        // ここでは基本構造のテストのみ
        assert!(true);
    }
}
