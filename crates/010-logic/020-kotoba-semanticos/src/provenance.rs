//! Provenance implementation for SemanticOS
//!
//! Records execution history in JSON-LD/PROV-O format.

use crate::actor::ActorTrait;
use crate::types::{Process, ProvenanceEvent, Resource};
use crate::Result;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tracing::info;

/// Provenance service for recording execution history
pub struct Provenance {
    /// Recorded events
    events: Vec<ProvenanceEvent>,
}

impl Provenance {
    /// Create a new provenance service
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Record a provenance event
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
        
        self.events.push(event);
        Ok(())
    }

    /// Get all recorded events
    pub fn events(&self) -> &[ProvenanceEvent] {
        &self.events
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
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

impl Default for Provenance {
    fn default() -> Self {
        Self::new()
    }
}

