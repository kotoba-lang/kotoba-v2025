//! Holochain DHT直接操作
//!
//! Holochain DHTを直接操作してMerkle DAGを構築する機能を提供します。

use crate::types::*;
use crate::Result;
use hdk::prelude::*;
use serde_json::Value;

/// JSON-LDデータをDHTエントリとして保存
pub async fn store_jsonld_entry(
    entry_type: &str,
    data: &Value,
) -> Result<EntryHash> {
    // JSON-LDデータをシリアライズ
    let entry_data = serde_json::to_vec(data)
        .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;

    // エントリタイプに応じたエントリを作成
    let entry = match entry_type {
        "Story" => {
            let story_entry: StoryEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&story_entry)?))
        }
        "Process" => {
            let process_entry: ProcessEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&process_entry)?))
        }
        "Provenance" => {
            let provenance_entry: ProvenanceEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&provenance_entry)?))
        }
        "Evolution" => {
            let evolution_entry: EvolutionEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&evolution_entry)?))
        }
        "MerkleNode" => {
            let merkle_entry: MerkleNodeEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&merkle_entry)?))
        }
        "Actor" => {
            let actor_entry: ActorEntry = serde_json::from_value(data.clone())
                .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
            Entry::App(EntryBytes::from(serde_json::to_vec(&actor_entry)?))
        }
        _ => {
            return Err(crate::HolochainKotobasosError::Dht(
                format!("Unknown entry type: {}", entry_type)
            ));
        }
    };

    // DHTに作成
    let entry_hash = create_entry(entry)
        .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to create entry: {}", e)))?;

    Ok(entry_hash)
}

/// DHTからJSON-LDデータを取得
pub async fn get_jsonld_entry(entry_hash: &EntryHash) -> Result<Value> {
    // DHTから取得
    let element = get(*entry_hash, GetOptions::default())
        .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to get entry: {}", e)))?;

    match element {
        Some(element) => {
            // エントリをJSON-LDに変換
            let entry = element.entry();
            match entry {
                Entry::App(entry_bytes) => {
                    let value: Value = serde_json::from_slice(entry_bytes.as_slice())
                        .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
                    Ok(value)
                }
                _ => Err(crate::HolochainKotobasosError::Dht(
                    "Entry is not an App entry".to_string()
                )),
            }
        }
        None => Err(crate::HolochainKotobasosError::Dht("Entry not found".to_string())),
    }
}

/// DHTクエリ実行
pub async fn query_dht(query: &DhtQuery) -> Result<Vec<(EntryHash, Value)>> {
    // TODO: 実際のDHTクエリ実装
    // Holochain v0.4ではquery APIを使用
    // 現在はプレースホルダー実装

    // エントリタイプに基づいてクエリ
    let mut results = Vec::new();

    // 簡易的な実装: すべてのエントリを取得してフィルタリング
    // 実際の実装では、より効率的なクエリ方法を使用する必要がある

    Ok(results)
}

/// Merkle DAG構造をDHT上に構築
pub async fn build_merkle_dag(
    root_data: &Value,
    links: &[(String, String)], // (parent_cid, child_cid)
) -> Result<String> {
    use crate::utils::jsonld_to_cid;

    // ルートノードのCIDを計算
    let root_cid = jsonld_to_cid(root_data)?;

    // ルートノードをMerkleNodeEntryとして保存
    let root_node = MerkleNodeEntry {
        id: root_cid.clone(),
        data_hash: root_cid.clone(),
        parent_links: Vec::new(),
        child_links: links.iter().map(|(_, child)| child.clone()).collect(),
        metadata: json!({}),
    };

    let root_data_value = serde_json::to_value(&root_node)?;
    store_jsonld_entry("MerkleNode", &root_data_value).await?;

    // 子ノードを処理
    for (parent_cid, child_cid) in links {
        // 子ノードのデータを取得（実際の実装では適切なデータソースから取得）
        // ここではプレースホルダー
        let child_node = MerkleNodeEntry {
            id: child_cid.clone(),
            data_hash: child_cid.clone(),
            parent_links: vec![parent_cid.clone()],
            child_links: Vec::new(),
            metadata: json!({}),
        };

        let child_data_value = serde_json::to_value(&child_node)?;
        store_jsonld_entry("MerkleNode", &child_data_value).await?;
    }

    Ok(root_cid)
}

/// CID（Content ID）からDHTエントリを解決
pub async fn resolve_cid(cid: &str) -> Result<Value> {
    // CIDからEntryHashへのマッピングが必要
    // 実際の実装では、CIDをキーとして使用するインデックスエントリを作成する必要がある
    // 現在はプレースホルダー

    // TODO: CIDインデックスエントリからEntryHashを検索
    // その後、get_jsonld_entryを使用してデータを取得

    Err(crate::HolochainKotobasosError::Dht("CID resolution not implemented yet".to_string()))
}

/// エントリにリンクを作成
pub async fn create_entry_link(
    base: &EntryHash,
    target: &EntryHash,
    tag: &str,
) -> Result<HeaderHash> {
    let link_tag = LinkTag::new(tag.as_bytes().to_vec());
    let header_hash = hdk::prelude::create_link(
        base.clone(),
        target.clone(),
        link_tag,
    )
    .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to create link: {}", e)))?;

    Ok(header_hash)
}

/// リンクを取得
pub async fn get_entry_links(
    base: &EntryHash,
    tag: Option<&str>,
) -> Result<Vec<Link>> {
    let link_tag = tag.map(|t| LinkTag::new(t.as_bytes().to_vec()));
    
    let links = hdk::prelude::get_links(
        base.clone(),
        link_tag.map(|lt| GetLinksInputBuilder::try_new(lt).unwrap().build()),
    )
    .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to get links: {}", e)))?;

    Ok(links.into_inner())
}

