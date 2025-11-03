//! Holochain DHT直接操作
//!
//! Holochain DHTを直接操作してMerkle DAGを構築する機能を提供します。

use crate::types::*;
use crate::Result;
use hdk::prelude::*;
use serde_json::Value;

/// JSON-LDデータをDHTエントリとして保存
/// CIDインデックスも自動的に作成します
pub async fn store_jsonld_entry(
    entry_type: &str,
    data: &Value,
) -> Result<EntryHash> {
    use crate::types::CidIndexEntry;
    use crate::utils::jsonld_to_cid;

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

    // CIDインデックスを作成
    let cid = jsonld_to_cid(data)?;
    let cid_index = CidIndexEntry {
        cid: cid.clone(),
        entry_hash,
        entry_type: entry_type.to_string(),
        created_at: chrono::Utc::now().timestamp(),
    };
    
    // CIDインデックスエントリを保存
    let cid_index_value = serde_json::to_value(&cid_index)?;
    let cid_index_entry = Entry::App(EntryBytes::from(serde_json::to_vec(&cid_index_value)?));
    let cid_index_hash = create_entry(cid_index_entry)
        .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to create CID index: {}", e)))?;

    // CIDからEntryHashへのリンクを作成（検索用）
    // CIDのハッシュをベースとしてリンクを作成
    let cid_bytes = cid.as_bytes();
    let cid_entry = Entry::App(EntryBytes::from(cid_bytes));
    let cid_hash = hdk::hash::hash_entry(cid_entry)?;
    
    // CIDハッシュからCIDインデックスエントリへのリンクを作成
    let _link_hash = create_link(
        cid_hash,
        cid_index_hash,
        LinkTag::new("cid_index".as_bytes().to_vec()),
    )
    .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to create CID link: {}", e)))?;

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
    use crate::types::CidIndexEntry;
    
    let mut results = Vec::new();

    // エントリタイプに基づいてクエリ
    // Holochain v0.4では、リンクとエントリタイプベースのクエリを使用
    
    // エントリタイプのルートハッシュを作成（簡易的な方法）
    // 実際の実装では、より効率的なインデックス構造を使用する必要がある
    let entry_type_hash = hdk::hash::hash_entry(
        Entry::App(EntryBytes::from(query.entry_type.as_bytes()))
    )?;

    // エントリタイプからリンクを取得
    let links = get_links(
        entry_type_hash,
        Some(LinkTypesFilter::single(
            holochain_zome_types::link::LinkType::new(0)
        )),
    )
    .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to get links: {}", e)))?;

    // リンクされたエントリを取得
    for link in links.into_inner() {
        let target_hash = link.target;
        match get(target_hash, GetOptions::default()) {
            Ok(Some(element)) => {
                if let Entry::App(entry_bytes) = element.entry() {
                    let entry_value: Value = serde_json::from_slice(entry_bytes.as_slice())
                        .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
                    
                    // フィルタ条件を適用
                    if apply_filters(&entry_value, &query.filters) {
                        results.push((target_hash, entry_value));
                    }
                }
            }
            Ok(None) => continue,
            Err(e) => {
                // エラーは無視して続行
                tracing::warn!("Failed to get entry: {}", e);
                continue;
            }
        }
    }

    // ページネーションを適用
    if let Some(pagination) = &query.pagination {
        let start = pagination.offset;
        let end = start + pagination.limit;
        if start < results.len() {
            results = results[start..end.min(results.len())].to_vec();
        } else {
            results.clear();
        }
    }

    Ok(results)
}

/// フィルタ条件を適用
fn apply_filters(entry_value: &Value, filters: &Value) -> bool {
    if let Some(filter_obj) = filters.as_object() {
        for (key, filter_value) in filter_obj {
            if let Some(entry_field) = entry_value.get(key) {
                if entry_field != filter_value {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    true
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
    use crate::types::CidIndexEntry;
    
    // CIDのハッシュを計算
    let cid_bytes = cid.as_bytes();
    let cid_entry = Entry::App(EntryBytes::from(cid_bytes));
    let cid_hash = hdk::hash::hash_entry(cid_entry)?;

    // CIDインデックスリンクを取得
    let links = get_links(
        cid_hash,
        Some(LinkTypesFilter::single(
            holochain_zome_types::link::LinkType::new(0)
        )),
    )
    .map_err(|e| crate::HolochainKotobasosError::Dht(format!("Failed to get CID links: {}", e)))?;

    // 最初のリンクからCIDインデックスエントリのEntryHashを取得
    if let Some(link) = links.into_inner().first() {
        let cid_index_hash = link.target;
        
        // CIDインデックスエントリを取得
        if let Ok(Some(element)) = get(cid_index_hash, GetOptions::default()) {
            if let Entry::App(entry_bytes) = element.entry() {
                let cid_index: CidIndexEntry = serde_json::from_slice(entry_bytes.as_slice())
                    .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
                
                // 実際のエントリを取得
                return get_jsonld_entry(&cid_index.entry_hash).await;
            }
        }
    }

    // フォールバック: CIDインデックスエントリを直接検索
    // すべてのCIDインデックスエントリを検索（非効率的だが動作する）
    let query = DhtQuery {
        entry_type: "CidIndex".to_string(),
        filters: serde_json::json!({
            "cid": cid
        }),
        pagination: None,
    };
    
    let results = query_dht(&query).await?;
    if let Some((_, index_value)) = results.first() {
        let cid_index: CidIndexEntry = serde_json::from_value(index_value.clone())?;
        return get_jsonld_entry(&cid_index.entry_hash).await;
    }

    Err(crate::HolochainKotobasosError::Dht(
        format!("CID not found: {}", cid)
    ))
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

