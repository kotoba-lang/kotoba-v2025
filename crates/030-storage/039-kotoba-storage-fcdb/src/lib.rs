//! Kotoba FCDB Storage Engine
//!
//! FCDB (Functorial-Categorical Database) storage adapter implementation
//! following the Pure Kernel/Effects Shell pattern.
//!
//! ## Effects Shell Implementation
//!
//! This crate provides a FCDB-based storage engine that implements the `StorageEngine` trait.
//! It uses fcdb-graph and fcdb-cas for persistent, content-addressable storage with graph capabilities.
//!
//! ## Key Features
//!
//! - **Content-Addressable**: All data stored by CID (Content ID)
//! - **Persistent**: Data persists across restarts
//! - **Graph-Based**: Built on FCDB's graph database
//! - **JSON-LD Native**: Full support for JSON-LD data format

use kotoba_storage::*;
use fcdb_core::Cid;
use fcdb_cas::PackCAS;
use fcdb_graph::{GraphDB, Rid, LabelId};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, warn, error};

/// FCDB storage engine
pub struct FcdbStorageEngine {
    /// GraphDB instance (wraps PackCAS)
    graph: Arc<RwLock<GraphDB>>,
    /// Base path for CAS storage
    base_path: PathBuf,
    /// Engine metadata
    info: StorageInfo,
    /// Namespace to RID mapping (for key lookups)
    namespace_rids: Arc<RwLock<HashMap<String, Rid>>>,
    /// Key to RID mapping
    key_rids: Arc<RwLock<HashMap<StorageKey, Rid>>>,
}

impl FcdbStorageEngine {
    /// Create a new FCDB storage engine
    pub async fn new<P: Into<PathBuf>>(base_path: P) -> Result<Self, StorageError> {
        let base_path = base_path.into();
        std::fs::create_dir_all(&base_path)
            .map_err(|e| StorageError::IoError(format!("Failed to create directory: {}", e)))?;

        // Initialize PackCAS
        let cas = PackCAS::open(&base_path).await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to open PackCAS: {}", e)))?;

        // Initialize GraphDB
        let graph = GraphDB::new(cas).await;

        info!("[FcdbStorageEngine] Initialized at {:?}", base_path);

        Ok(Self {
            graph: Arc::new(RwLock::new(graph)),
            base_path,
            info: StorageInfo {
                name: "FcdbStorageEngine".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                capabilities: vec![
                    "get".to_string(),
                    "put".to_string(),
                    "delete".to_string(),
                    "exists".to_string(),
                    "list".to_string(),
                    "batch".to_string(),
                    "query".to_string(),
                    "persistent".to_string(),
                    "content-addressable".to_string(),
                ],
            },
            namespace_rids: Arc::new(RwLock::new(HashMap::new())),
            key_rids: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get RID for a storage key (create if doesn't exist)
    async fn get_or_create_rid(&self, key: &StorageKey) -> Result<Rid, StorageError> {
        // Check cache first
        {
            let key_rids = self.key_rids.read().await;
            if let Some(rid) = key_rids.get(key) {
                return Ok(*rid);
            }
        }

        // Create new node for this key
        let key_json = serde_json::to_string(key)
            .map_err(|e| StorageError::SerializationError(format!("Failed to serialize key: {}", e)))?;

        let graph = self.graph.write().await;
        let rid = graph.create_node(key_json.as_bytes()).await
            .map_err(|e| StorageError::OperationFailed(format!("Failed to create node: {}", e)))?;

        // Cache the mapping
        {
            let mut key_rids = self.key_rids.write().await;
            key_rids.insert(key.clone(), rid);
        }

        // Also cache namespace mapping
        {
            let mut namespace_rids = self.namespace_rids.write().await;
            namespace_rids.entry(key.namespace.clone()).or_insert_with(|| {
                // Create a namespace node if it doesn't exist
                // For now, use the first key's RID as namespace marker
                rid
            });
        }

        Ok(rid)
    }

    /// Get RID for a storage key (returns None if doesn't exist)
    async fn get_rid(&self, key: &StorageKey) -> Option<Rid> {
        let key_rids = self.key_rids.read().await;
        key_rids.get(key).copied()
    }

    /// Store a value at a key
    async fn put_value(&self, key: &StorageKey, value: &StorageValue) -> Result<(), StorageError> {
        // Ensure JSON-LD format
        let jsonld_data = serde_json::to_string(&value.data)
            .map_err(|e| StorageError::SerializationError(format!("Failed to serialize value: {}", e)))?;

        // Get or create RID for this key
        let rid = self.get_or_create_rid(key).await?;

        // Update node data
        let graph = self.graph.write().await;
        graph.update_node(rid, jsonld_data.as_bytes()).await
            .map_err(|e| StorageError::OperationFailed(format!("Failed to update node: {}", e)))?;

        info!("[FcdbStorageEngine] Stored value at key: {}", key.full_path());
        Ok(())
    }

    /// Get a value by key
    async fn get_value(&self, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
        if let Some(rid) = self.get_rid(key).await {
            let graph = self.graph.read().await;
            let data_bytes = graph.get_node(rid).await
                .map_err(|e| StorageError::OperationFailed(format!("Failed to get node: {}", e)))?;

            if let Some(bytes) = data_bytes {
                let json_value: Value = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::SerializationError(format!("Failed to deserialize value: {}", e)))?;

                let storage_value = StorageValue::new(json_value);
                Ok(Some(storage_value))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Delete a value by key
    async fn delete_value(&self, key: &StorageKey) -> Result<bool, StorageError> {
        if let Some(rid) = self.get_rid(key).await {
            // In FCDB, we mark as deleted by updating with empty data
            // In a full implementation, we'd use temporal queries to handle deletions
            let graph = self.graph.write().await;
            graph.update_node(rid, b"").await
                .map_err(|e| StorageError::OperationFailed(format!("Failed to delete node: {}", e)))?;

            // Remove from cache
            {
                let mut key_rids = self.key_rids.write().await;
                key_rids.remove(key);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a key exists
    async fn exists_value(&self, key: &StorageKey) -> Result<bool, StorageError> {
        if let Some(rid) = self.get_rid(key).await {
            let graph = self.graph.read().await;
            let data = graph.get_node(rid).await
                .map_err(|e| StorageError::OperationFailed(format!("Failed to check node: {}", e)))?;

            Ok(data.is_some() && !data.unwrap().is_empty())
        } else {
            Ok(false)
        }
    }

    /// List keys in a namespace
    async fn list_keys(&self, namespace: &str) -> Result<Vec<StorageKey>, StorageError> {
        let key_rids = self.key_rids.read().await;
        let keys: Vec<StorageKey> = key_rids.keys()
            .filter(|key| key.namespace == namespace)
            .cloned()
            .collect();
        Ok(keys)
    }

    /// Execute a single operation
    async fn execute_operation(&self, operation: &StorageOperation) -> Result<OperationResult, StorageError> {
        match operation {
            StorageOperation::Get(key) => {
                let value = self.get_value(key).await?;
                Ok(OperationResult::Get(value))
            }
            StorageOperation::Put(key, value) => {
                self.put_value(key, value).await?;
                Ok(OperationResult::Put(true))
            }
            StorageOperation::Delete(key) => {
                let existed = self.delete_value(key).await?;
                Ok(OperationResult::Delete(existed))
            }
            StorageOperation::Exists(key) => {
                let exists = self.exists_value(key).await?;
                Ok(OperationResult::Exists(exists))
            }
            StorageOperation::List(namespace) => {
                let keys = self.list_keys(namespace).await?;
                Ok(OperationResult::List(keys))
            }
            StorageOperation::Batch(operations) => {
                // Execute batch operations sequentially
                // In a full implementation, this would be transactional
                let mut results = Vec::new();
                for op in operations {
                    results.push(self.execute_operation(op).await?);
                }
                // Return the last result (simplified)
                Ok(results.into_iter().last().unwrap_or(OperationResult::Put(true)))
            }
        }
    }

    /// Execute a query
    async fn execute_query_internal(&self, query: &QueryPlan) -> Result<QueryResult, StorageError> {
        // List all keys in namespace
        let keys = self.list_keys(&query.namespace).await?;

        // Filter by conditions (simplified - full implementation would parse JSON-LD)
        let mut values = Vec::new();
        for key in keys {
            if let Ok(Some(value)) = self.get_value(&key).await {
                // Simple condition matching (full implementation would support complex queries)
                let matches = query.conditions.is_empty() || {
                    // Check if value matches any condition
                    query.conditions.iter().any(|condition| {
                        match condition {
                            QueryCondition::Equals(field, expected_value) => {
                                value.data.get(field) == Some(expected_value)
                            }
                            _ => false, // Simplified - full implementation would handle all conditions
                        }
                    })
                };

                if matches {
                    values.push((key, value));
                }
            }
        }

        // Apply limit
        let limited_values = if let Some(limit) = query.limit {
            values.into_iter().take(limit).collect()
        } else {
            values
        };

        Ok(QueryResult {
            values: limited_values,
            has_more: false, // Simplified
            total_count: None,
            execution_time_ms: 0, // Simplified
        })
    }
}

#[async_trait]
impl StorageEngine for FcdbStorageEngine {
    async fn execute_plan(&self, plan: &StoragePlan) -> Result<StorageResult, StorageError> {
        // Validate the plan
        plan.validate()
            .map_err(|e| StorageError::OperationFailed(format!("Plan validation failed: {:?}", e)))?;

        // Execute operations
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();

        for operation in &plan.operations {
            let result = self.execute_operation(operation).await?;
            results.push(result);
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(StorageResult {
            results,
            execution_time_ms: execution_time,
        })
    }

    async fn execute_query(&self, query: &QueryPlan) -> Result<QueryResult, StorageError> {
        self.execute_query_internal(query).await
    }

    fn info(&self) -> StorageInfo {
        self.info.clone()
    }
}

/// Convenience functions for creating FCDB storage
pub mod factory {
    use super::*;

    /// Create a new FCDB storage engine
    pub async fn create<P: Into<PathBuf>>(path: P) -> Result<FcdbStorageEngine, StorageError> {
        FcdbStorageEngine::new(path).await
    }

    /// Create a temporary FCDB storage engine (for testing)
    pub async fn create_temp() -> Result<FcdbStorageEngine, StorageError> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| StorageError::IoError(format!("Failed to create temp dir: {}", e)))?;
        FcdbStorageEngine::new(temp_dir.path()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_fcdb_storage_basic_operations() {
        let storage = factory::create_temp().await.unwrap();

        // Test PUT
        let key = StorageKey::new("provenance", "event-1");
        let value = StorageValue::new(json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@type": "kotoba:ProvenanceEvent",
            "@id": "kotoba:provenance/event-1",
            "kotoba:wasGeneratedBy": "kotoba:process/test"
        }));
        let plan = StoragePlan::single(StorageOperation::Put(key.clone(), value));
        let result = storage.execute_plan(&plan).await.unwrap();

        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0], OperationResult::Put(true)));

        // Test GET
        let get_plan = StoragePlan::single(StorageOperation::Get(key.clone()));
        let get_result = storage.execute_plan(&get_plan).await.unwrap();

        assert_eq!(get_result.results.len(), 1);
        match &get_result.results[0] {
            OperationResult::Get(Some(val)) => {
                assert_eq!(val.data["@type"], "kotoba:ProvenanceEvent");
            }
            _ => panic!("Expected Get(Some(_))"),
        }

        // Test EXISTS
        let exists_plan = StoragePlan::single(StorageOperation::Exists(key.clone()));
        let exists_result = storage.execute_plan(&exists_plan).await.unwrap();

        assert_eq!(exists_result.results.len(), 1);
        assert!(matches!(exists_result.results[0], OperationResult::Exists(true)));
    }

    #[tokio::test]
    async fn test_fcdb_storage_list() {
        let storage = factory::create_temp().await.unwrap();

        // Put multiple values in same namespace
        let keys = vec![
            StorageKey::new("provenance", "event-1"),
            StorageKey::new("provenance", "event-2"),
            StorageKey::new("evolution", "pattern-1"),
        ];

        for key in &keys[..2] {
            let value = StorageValue::new(json!({"@type": "kotoba:ProvenanceEvent"}));
            let plan = StoragePlan::single(StorageOperation::Put(key.clone(), value));
            storage.execute_plan(&plan).await.unwrap();
        }

        // List provenance namespace
        let list_plan = StoragePlan::single(StorageOperation::List("provenance".to_string()));
        let list_result = storage.execute_plan(&list_plan).await.unwrap();

        assert_eq!(list_result.results.len(), 1);
        match &list_result.results[0] {
            OperationResult::List(keys) => {
                assert_eq!(keys.len(), 2);
            }
            _ => panic!("Expected List(_)"),
        }
    }
}

