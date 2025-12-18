//! DHTクエリキャッシュとCIDインデックスキャッシュ
//!
//! パフォーマンス最適化のためのキャッシュ機能を提供します。

use crate::types::*;
use hdk::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// キャッシュエントリ
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// CIDインデックスキャッシュ
pub struct CidIndexCache {
    /// CIDからEntryHashへのマッピング
    cid_to_entry: Arc<RwLock<HashMap<String, EntryHash>>>,
    /// キャッシュのTTL（デフォルト: 5分）
    ttl: Duration,
}

impl CidIndexCache {
    /// 新しいCIDインデックスキャッシュを作成
    pub fn new(ttl: Duration) -> Self {
        Self {
            cid_to_entry: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// デフォルト設定でCIDインデックスキャッシュを作成（TTL: 5分）
    pub fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }

    /// CIDからEntryHashを取得（キャッシュから）
    pub async fn get(&self, cid: &str) -> Option<EntryHash> {
        let cache = self.cid_to_entry.read().await;
        cache.get(cid).copied()
    }

    /// CIDとEntryHashのマッピングをキャッシュに保存
    pub async fn put(&self, cid: String, entry_hash: EntryHash) {
        let mut cache = self.cid_to_entry.write().await;
        cache.insert(cid, entry_hash);
    }

    /// キャッシュをクリア
    pub async fn clear(&self) {
        let mut cache = self.cid_to_entry.write().await;
        cache.clear();
    }

    /// 期限切れのエントリを削除
    pub async fn evict_expired(&self) {
        // 現在の実装では、TTLに基づいた自動削除は行わない
        // 必要に応じて、定期的なクリーンアップタスクを実装
    }
}

/// DHTクエリ結果キャッシュ
pub struct DhtQueryCache {
    /// クエリハッシュから結果へのマッピング
    query_results: Arc<RwLock<HashMap<String, CacheEntry<Vec<(EntryHash, Value)>>>>>,
    /// キャッシュのTTL（デフォルト: 1分）
    ttl: Duration,
}

impl DhtQueryCache {
    /// 新しいDHTクエリキャッシュを作成
    pub fn new(ttl: Duration) -> Self {
        Self {
            query_results: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// デフォルト設定でDHTクエリキャッシュを作成（TTL: 1分）
    pub fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }

    /// クエリキーを生成
    fn query_key(&self, query: &DhtQuery) -> String {
        format!("{}:{}", query.entry_type, serde_json::to_string(&query.filters).unwrap_or_default())
    }

    /// クエリ結果を取得（キャッシュから）
    pub async fn get(&self, query: &DhtQuery) -> Option<Vec<(EntryHash, Value)>> {
        let key = self.query_key(query);
        let cache = self.query_results.read().await;
        
        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        
        None
    }

    /// クエリ結果をキャッシュに保存
    pub async fn put(&self, query: &DhtQuery, results: Vec<(EntryHash, Value)>) {
        let key = self.query_key(query);
        let mut cache = self.query_results.write().await;
        cache.insert(key, CacheEntry::new(results, self.ttl));
    }

    /// キャッシュをクリア
    pub async fn clear(&self) {
        let mut cache = self.query_results.write().await;
        cache.clear();
    }

    /// 期限切れのエントリを削除
    pub async fn evict_expired(&self) {
        let mut cache = self.query_results.write().await;
        cache.retain(|_, entry| !entry.is_expired());
    }
}

/// グローバルキャッシュマネージャー
pub struct CacheManager {
    cid_index: CidIndexCache,
    query_cache: DhtQueryCache,
}

impl CacheManager {
    /// 新しいキャッシュマネージャーを作成
    pub fn new(cid_ttl: Duration, query_ttl: Duration) -> Self {
        Self {
            cid_index: CidIndexCache::new(cid_ttl),
            query_cache: DhtQueryCache::new(query_ttl),
        }
    }

    /// デフォルト設定でキャッシュマネージャーを作成
    pub fn default() -> Self {
        Self::new(
            Duration::from_secs(300), // CID: 5分
            Duration::from_secs(60),  // Query: 1分
        )
    }

    /// CIDインデックスキャッシュを取得
    pub fn cid_index(&self) -> &CidIndexCache {
        &self.cid_index
    }

    /// DHTクエリキャッシュを取得
    pub fn query_cache(&self) -> &DhtQueryCache {
        &self.query_cache
    }

    /// すべてのキャッシュをクリア
    pub async fn clear_all(&self) {
        self.cid_index.clear().await;
        self.query_cache.clear().await;
    }

    /// 期限切れのエントリを削除
    pub async fn evict_expired(&self) {
        self.cid_index.evict_expired().await;
        self.query_cache.evict_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cid_index_cache() {
        let cache = CidIndexCache::default();
        let cid = "cid:test123";
        let entry_hash = EntryHash::from_raw_39(&[0u8; 39]).unwrap();

        // キャッシュに保存
        cache.put(cid.to_string(), entry_hash).await;

        // キャッシュから取得
        let result = cache.get(cid).await;
        assert_eq!(result, Some(entry_hash));

        // 存在しないCID
        let result2 = cache.get("cid:nonexistent").await;
        assert_eq!(result2, None);
    }

    #[tokio::test]
    async fn test_dht_query_cache() {
        use serde_json::json;

        let cache = DhtQueryCache::default();
        let query = DhtQuery {
            entry_type: "Story".to_string(),
            filters: json!({"id": "story:1"}),
            pagination: None,
        };

        let results = vec![
            (EntryHash::from_raw_39(&[0u8; 39]).unwrap(), json!({"id": "story:1"})),
        ];

        // キャッシュに保存
        cache.put(&query, results.clone()).await;

        // キャッシュから取得
        let cached = cache.get(&query).await;
        assert_eq!(cached, Some(results));
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let manager = CacheManager::default();
        
        // CIDキャッシュのテスト
        let cid = "cid:test";
        let entry_hash = EntryHash::from_raw_39(&[0u8; 39]).unwrap();
        manager.cid_index().put(cid.to_string(), entry_hash).await;
        assert_eq!(manager.cid_index().get(cid).await, Some(entry_hash));

        // クエリキャッシュのテスト
        use serde_json::json;
        let query = DhtQuery {
            entry_type: "Process".to_string(),
            filters: json!({}),
            pagination: None,
        };
        let results = vec![];
        manager.query_cache().put(&query, results.clone()).await;
        assert_eq!(manager.query_cache().get(&query).await, Some(results));

        // クリア
        manager.clear_all().await;
        assert_eq!(manager.cid_index().get(cid).await, None);
    }
}

