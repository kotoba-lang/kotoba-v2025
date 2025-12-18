//! Provenance implementation for KotobaOS
//!
//! Records execution history in JSON-LD/PROV-O format with persistent storage.

use crate::actor::ActorTrait;
use crate::types::{Process, ProvenanceEvent, Resource};
use crate::Result;
use chrono::Utc;
use kotoba_storage::{StorageEngine, StorageKey, StorageValue, StorageOperation, StoragePlan};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};

/// Provenance service for recording execution history
pub struct Provenance {
    /// Recorded events (in-memory cache)
    events: Vec<ProvenanceEvent>,
    /// Storage engine for persistence
    storage: Arc<dyn StorageEngine>,
}

impl Provenance {
    /// Create a new provenance service with storage
    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self {
            events: Vec::new(),
            storage,
        }
    }

    /// Load events from storage
    pub async fn load_from_storage(&mut self) -> Result<()> {
        info!("[Provenance] Loading events from storage");

        // List all provenance events from storage
        let list_op = StorageOperation::List("provenance".to_string());
        let plan = StoragePlan::single(list_op);
        
        match self.storage.execute_plan(&plan).await {
            Ok(result) => {
                if let Some(kotoba_storage::OperationResult::List(keys)) = result.results.first() {
                    // Load each event
                    for key in keys {
                        let get_op = StorageOperation::Get(key.clone());
                        let get_plan = StoragePlan::single(get_op);
                        
                        if let Ok(get_result) = self.storage.execute_plan(&get_plan).await {
                            if let Some(kotoba_storage::OperationResult::Get(Some(value))) = get_result.results.first() {
                                // Deserialize event from storage value
                                if let Ok(event) = serde_json::from_value::<ProvenanceEvent>(value.data.clone()) {
                                    self.events.push(event);
                                }
                            }
                        }
                    }
                }
                info!("[Provenance] Loaded {} events from storage", self.events.len());
            }
            Err(e) => {
                warn!("[Provenance] Failed to load events from storage: {}", e);
                // Continue with empty events - this is not fatal
            }
        }

        Ok(())
    }

    /// Record a provenance event (automatically persists to storage)
    pub async fn record(
        &mut self,
        process: &Process,
        actor: &Arc<dyn ActorTrait>,
        result: &Resource,
    ) -> Result<()> {
        let event = ProvenanceEvent {
            id: format!("kotoba:provenance/event-{}", uuid::Uuid::new_v4()),
            type_: "kotoba:ProvenanceEvent".to_string(),
            context: process.context.clone(),
            was_generated_by: process.id.clone(),
            was_associated_with: Some(actor.id().to_string()),
            used: process.used.clone(),
            generated: process.generated.clone(),
            ended_at_time: Utc::now().to_rfc3339(),
            additional: {
                let mut additional = std::collections::HashMap::new();
                additional.insert(
                    "prov:startedAtTime".to_string(),
                    json!(Utc::now().to_rfc3339()),
                );
                additional.insert(
                    "prov:result".to_string(),
                    json!(result.id),
                );
                additional
            },
        };

        info!("[Provenance] Recording event: {} for process: {}", 
              event.id, process.id);
        
        // Add to in-memory cache
        self.events.push(event.clone());

        // Persist to storage immediately
        self.persist_event(&event).await?;

        Ok(())
    }

    /// Persist a single event to storage
    async fn persist_event(&self, event: &ProvenanceEvent) -> Result<()> {
        // Create storage key
        let key = StorageKey::new("provenance", &event.id);

        // Convert event to JSON-LD storage value
        let event_json = serde_json::to_value(event)
            .map_err(|e| crate::KotobaOsError::Other(anyhow::anyhow!("Failed to serialize event: {}", e)))?;

        let storage_value = StorageValue::new(event_json);

        // Store in storage
        let put_op = StorageOperation::Put(key, storage_value);
        let plan = StoragePlan::single(put_op);

        self.storage.execute_plan(&plan).await
            .map_err(|e| crate::KotobaOsError::Other(anyhow::anyhow!("Failed to persist event: {}", e)))?;

        info!("[Provenance] Persisted event {} to storage", event.id);
        Ok(())
    }

    /// Get all recorded events
    pub fn events(&self) -> &[ProvenanceEvent] {
        &self.events
    }

    /// Clear all events (both memory and storage)
    pub async fn clear(&mut self) -> Result<()> {
        // List all provenance keys
        let list_op = StorageOperation::List("provenance".to_string());
        let list_plan = StoragePlan::single(list_op);
        
        if let Ok(result) = self.storage.execute_plan(&list_plan).await {
            if let Some(kotoba_storage::OperationResult::List(keys)) = result.results.first() {
                // Delete each key
                for key in keys {
                    let delete_op = StorageOperation::Delete(key.clone());
                    let delete_plan = StoragePlan::single(delete_op);
                    let _ = self.storage.execute_plan(&delete_plan).await;
                }
            }
        }

        self.events.clear();
        Ok(())
    }

    /// Convert events to JSON-LD format
    pub fn to_jsonld(&self) -> serde_json::Value {
        json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "prov": "http://www.w3.org/ns/prov#",
                "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
            },
            "@graph": self.events
        })
    }
}
