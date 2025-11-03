//! Mediator implementation for KotobaOS
//!
//! The Mediator selects appropriate actors for process execution based on
//! SHACL-based capability matching and semantic similarity.

use crate::actor::{Actor, ActorTrait};
use crate::types::Process;
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Mediator manages actor registration and selection
pub struct Mediator {
    /// Registered actors by ID
    actors: HashMap<String, Arc<dyn ActorTrait>>,

    /// Actors by capability IRI
    actors_by_capability: HashMap<String, Vec<String>>,
}

impl Mediator {
    /// Create a new mediator
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            actors_by_capability: HashMap::new(),
        }
    }

    /// Register an actor with the mediator
    pub fn register_actor<A: ActorTrait + 'static>(&mut self, actor: A) {
        let id = actor.id().to_string();
        let capability = actor.capability().to_string();
        
        info!("[Mediator] Registering actor: {} with capability: {}", id, capability);
        
        let actor_arc = Arc::new(actor);
        
        // Store by ID
        self.actors.insert(id.clone(), actor_arc);
        
        // Index by capability
        self.actors_by_capability
            .entry(capability)
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// Select an actor for a given process
    /// 
    /// Current implementation uses simple mapping based on `performedBy`.
    /// Future versions will use SHACL-based semantic matching.
    pub async fn select_actor(&self, process: &Process) -> Result<Arc<dyn ActorTrait>> {
        // First, try to find actor by performedBy IRI
        if let Some(actor) = self.actors.get(&process.performed_by) {
            info!("[Mediator] Selected actor \"{}\" for process \"{}\"", 
                  actor.id(), 
                  process.label.as_deref().unwrap_or(&process.id));
            return Ok(Arc::clone(actor));
        }

        // Fallback: try to find by capability
        // Extract capability from process (if available in additional properties)
        if let Some(capability_value) = process.additional.get("kotoba:capability") {
            if let Some(capability) = capability_value.as_str() {
                if let Some(actor_ids) = self.actors_by_capability.get(capability) {
                    if let Some(first_id) = actor_ids.first() {
                        if let Some(actor) = self.actors.get(first_id) {
                            warn!("[Mediator] Selected actor \"{}\" by capability \"{}\" for process \"{}\"",
                                  actor.id(),
                                  capability,
                                  process.label.as_deref().unwrap_or(&process.id));
                            return Ok(Arc::clone(actor));
                        }
                    }
                }
            }
        }

        // Final fallback: return first available actor
        if let Some(first_actor) = self.actors.values().next() {
            warn!("[Mediator] No specific actor found. Falling back to default actor \"{}\"",
                  first_actor.id());
            return Ok(Arc::clone(first_actor));
        }

        Err(crate::KotobaOsError::ActorSelection(format!(
            "No suitable actor found for process \"{}\" and no fallback available",
            process.label.as_deref().unwrap_or(&process.id)
        )))
    }

    /// Get all registered actor IDs
    pub fn actor_ids(&self) -> Vec<String> {
        self.actors.keys().cloned().collect()
    }

    /// Check if an actor is registered
    pub fn has_actor(&self, id: &str) -> bool {
        self.actors.contains_key(id)
    }
}

impl Default for Mediator {
    fn default() -> Self {
        Self::new()
    }
}

