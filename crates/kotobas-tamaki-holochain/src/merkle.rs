//! DHT上でのMerkle DAG実装
//!
//! Merkle DAG構造をDHT上に実装します。

use crate::dht::{get_jsonld_entry, store_jsonld_entry};
use crate::types::MerkleNodeEntry;
use crate::Result;
use hdk::prelude::*;
use serde_json::{json, Value};

/// Merkle DAGノード
#[derive(Debug, Clone)]
pub struct MerkleNode {
    /// ノードID（CID）
    pub id: String,
    /// データハッシュ
    pub data_hash: String,
    /// 親ノードへのリンク（CIDs）
    pub parent_links: Vec<String>,
    /// 子ノードへのリンク（CIDs）
    pub child_links: Vec<String>,
    /// メタデータ
    pub metadata: Value,
    /// DHT上のEntryHash
    pub entry_hash: Option<EntryHash>,
}

impl MerkleNode {
    /// 新しいMerkleノードを作成
    pub fn new(id: String, data_hash: String) -> Self {
        Self {
            id,
            data_hash,
            parent_links: Vec::new(),
            child_links: Vec::new(),
            metadata: json!({}),
            entry_hash: None,
        }
    }

    /// DHTに保存
    pub async fn save_to_dht(&mut self) -> Result<()> {
        let entry = MerkleNodeEntry {
            id: self.id.clone(),
            data_hash: self.data_hash.clone(),
            parent_links: self.parent_links.clone(),
            child_links: self.child_links.clone(),
            metadata: self.metadata.clone(),
        };

        let entry_value = serde_json::to_value(&entry)?;
        let entry_hash = store_jsonld_entry("MerkleNode", &entry_value).await?;
        self.entry_hash = Some(entry_hash);
        Ok(())
    }

    /// DHTから読み込み
    pub async fn load_from_dht(entry_hash: &EntryHash) -> Result<Self> {
        let entry_value = get_jsonld_entry(entry_hash).await?;
        let entry: MerkleNodeEntry = serde_json::from_value(entry_value)?;

        Ok(Self {
            id: entry.id,
            data_hash: entry.data_hash,
            parent_links: entry.parent_links,
            child_links: entry.child_links,
            metadata: entry.metadata,
            entry_hash: Some(*entry_hash),
        })
    }
}

/// Merkleノード間のリンクを作成
pub async fn link_nodes(
    parent: &mut MerkleNode,
    child: &mut MerkleNode,
) -> Result<()> {
    use crate::dht::{create_entry_link, get_jsonld_entry, store_jsonld_entry};
    use crate::types::MerkleNodeEntry;

    // 親ノードの子リンクに追加
    if !parent.child_links.contains(&child.id) {
        parent.child_links.push(child.id.clone());
    }

    // 子ノードの親リンクに追加
    if !child.parent_links.contains(&parent.id) {
        child.parent_links.push(parent.id.clone());
    }

    // ノードを更新してDHTに保存
    if let Some(parent_hash) = parent.entry_hash {
        // 親ノードを更新
        let parent_entry = MerkleNodeEntry {
            id: parent.id.clone(),
            data_hash: parent.data_hash.clone(),
            parent_links: parent.parent_links.clone(),
            child_links: parent.child_links.clone(),
            metadata: parent.metadata.clone(),
        };
        let parent_entry_value = serde_json::to_value(&parent_entry)?;
        let new_parent_hash = store_jsonld_entry("MerkleNode", &parent_entry_value).await?;
        parent.entry_hash = Some(new_parent_hash);
    }

    if let Some(child_hash) = child.entry_hash {
        // 子ノードを更新
        let child_entry = MerkleNodeEntry {
            id: child.id.clone(),
            data_hash: child.data_hash.clone(),
            parent_links: child.parent_links.clone(),
            child_links: child.child_links.clone(),
            metadata: child.metadata.clone(),
        };
        let child_entry_value = serde_json::to_value(&child_entry)?;
        let new_child_hash = store_jsonld_entry("MerkleNode", &child_entry_value).await?;
        child.entry_hash = Some(new_child_hash);
    }

    // DHTリンクを作成（検索用）
    if let Some(parent_hash) = parent.entry_hash {
        if let Some(child_hash) = child.entry_hash {
            create_entry_link(&parent_hash, &child_hash, "merkle_child").await?;
            create_entry_link(&child_hash, &parent_hash, "merkle_parent").await?;
        }
    }

    Ok(())
}

/// Merkleパスの検証
pub async fn verify_merkle_path(
    root_cid: &str,
    leaf_cid: &str,
    path: &[String], // パス上のCIDs
) -> Result<bool> {
    // TODO: Merkleパスの検証ロジックを実装
    // 実際の実装では、パス上の各ノードを検証して、ハッシュチェーンが正しいことを確認

    // プレースホルダー実装
    Ok(true)
}

/// DAGの走査（深さ優先探索）
pub async fn traverse_dag(
    root_cid: &str,
    visitor: &mut dyn FnMut(&MerkleNode) -> Result<bool>,
) -> Result<()> {
    use std::collections::HashSet;

    let mut visited = HashSet::new();
    let mut stack = vec![root_cid.to_string()];

    while let Some(cid) = stack.pop() {
        if visited.contains(&cid) {
            continue;
        }
        visited.insert(cid.clone());

        // ノードを取得
        let node = get_merkle_node_by_cid(&cid).await?;

        // ビジターを呼び出し
        let should_continue = visitor(&node)?;
        if !should_continue {
            break;
        }

        // 子ノードをスタックに追加
        for child_cid in &node.child_links {
            if !visited.contains(child_cid) {
                stack.push(child_cid.clone());
            }
        }
    }

    Ok(())
}

/// CIDからMerkleノードを取得
pub async fn get_merkle_node_by_cid(cid: &str) -> Result<MerkleNode> {
    // CIDからEntryHashへの解決が必要
    // 実際の実装では、CIDインデックスエントリを使用
    use crate::dht::resolve_cid;
    
    let entry_value = resolve_cid(cid).await?;
    let entry: MerkleNodeEntry = serde_json::from_value(entry_value)?;

    // EntryHashを取得する必要があるが、現在はCID解決が未実装
    // プレースホルダー実装
    Ok(MerkleNode {
        id: entry.id,
        data_hash: entry.data_hash,
        parent_links: entry.parent_links,
        child_links: entry.child_links,
        metadata: entry.metadata,
        entry_hash: None,
    })
}

/// Merkle DAGのルートノードを作成
pub async fn create_root_node(data: &Value) -> Result<MerkleNode> {
    use crate::utils::jsonld_to_cid;

    let cid = jsonld_to_cid(data)?;
    let data_hash = cid.clone();

    let mut node = MerkleNode::new(cid, data_hash);
    node.metadata = json!({
        "created_at": chrono::Utc::now().timestamp(),
        "is_root": true,
    });
    node.save_to_dht().await?;

    Ok(node)
}

