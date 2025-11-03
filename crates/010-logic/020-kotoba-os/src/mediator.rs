//! Mediator implementation for KotobaOS
//!
//! The Mediator selects appropriate actors for process execution based on
//! SHACL-based capability matching and semantic similarity.

use crate::actor::ActorTrait;
use crate::types::Process;
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Actor selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Direct mapping by performedBy IRI
    Direct,
    /// Capability-based matching
    Capability,
    /// SHACL-based semantic matching (requires reasoning feature)
    #[cfg(feature = "reasoning")]
    ShaclSemantic,
}

/// Mediator manages actor registration and selection
pub struct Mediator {
    /// Registered actors by ID
    actors: HashMap<String, Arc<dyn ActorTrait>>,

    /// Actors by capability IRI
    actors_by_capability: HashMap<String, Vec<String>>,

    /// Selection strategy
    strategy: SelectionStrategy,
}

impl Mediator {
    /// Create a new mediator
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            actors_by_capability: HashMap::new(),
            strategy: SelectionStrategy::Direct,
        }
    }

    /// Create a mediator with selection strategy
    pub fn with_strategy(strategy: SelectionStrategy) -> Self {
        Self {
            actors: HashMap::new(),
            actors_by_capability: HashMap::new(),
            strategy,
        }
    }

    /// Set selection strategy
    pub fn set_strategy(&mut self, strategy: SelectionStrategy) {
        self.strategy = strategy;
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
    /// Uses different strategies based on configuration:
    /// - Direct: Simple mapping by performedBy IRI
    /// - Capability: Match by capability IRI
    /// - ShaclSemantic: SHACL-based semantic matching (requires reasoning feature)
    pub async fn select_actor(&self, process: &Process) -> Result<Arc<dyn ActorTrait>> {
        match self.strategy {
            SelectionStrategy::Direct => {
                self.select_by_direct(process).await
            }
            SelectionStrategy::Capability => {
                self.select_by_capability(process).await
            }
            #[cfg(feature = "reasoning")]
            SelectionStrategy::ShaclSemantic => {
                self.select_by_shacl_semantic(process).await
            }
        }
    }

    /// Select actor by direct IRI mapping
    async fn select_by_direct(&self, process: &Process) -> Result<Arc<dyn ActorTrait>> {
        // First, try to find actor by performedBy IRI
        if let Some(actor) = self.actors.get(&process.performed_by) {
            info!("[Mediator] Selected actor \"{}\" for process \"{}\" (direct)", 
                  actor.id(), 
                  process.label.as_deref().unwrap_or(&process.id));
            return Ok(Arc::clone(actor));
        }

        // Fallback to capability matching
        self.select_by_capability(process).await
    }

    /// Select actor by capability matching
    async fn select_by_capability(&self, process: &Process) -> Result<Arc<dyn ActorTrait>> {
        // Try to find by capability from process properties
        if let Some(capability_value) = process.additional.get("kotoba:capability") {
            if let Some(capability) = capability_value.as_str() {
                if let Some(actor_ids) = self.actors_by_capability.get(capability) {
                    if let Some(first_id) = actor_ids.first() {
                        if let Some(actor) = self.actors.get(first_id) {
                            info!("[Mediator] Selected actor \"{}\" by capability \"{}\" for process \"{}\"",
                                  actor.id(),
                                  capability,
                                  process.label.as_deref().unwrap_or(&process.id));
                            return Ok(Arc::clone(actor));
                        }
                    }
                }
            }
        }

        // Try capability matching with performedBy
        for (capability, actor_ids) in &self.actors_by_capability {
            if process.performed_by.contains(capability) || capability.contains(&process.performed_by) {
                if let Some(first_id) = actor_ids.first() {
                    if let Some(actor) = self.actors.get(first_id) {
                        warn!("[Mediator] Selected actor \"{}\" by partial capability match \"{}\" for process \"{}\"",
                              actor.id(),
                              capability,
                              process.label.as_deref().unwrap_or(&process.id));
                        return Ok(Arc::clone(actor));
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

    /// Select actor using SHACL-based semantic matching
    #[cfg(feature = "reasoning")]
    async fn select_by_shacl_semantic(&self, process: &Process) -> Result<Arc<dyn ActorTrait>> {
        use crate::shacl_validator::ShaclValidator;
        use serde_json::json;

        // First try direct matching
        if let Ok(actor) = self.select_by_direct(process).await {
            return Ok(actor);
        }

        // Calculate compatibility scores for all actors
        let mut scored_actors: Vec<(f64, Arc<dyn ActorTrait>)> = Vec::new();

        for actor in self.actors.values() {
            let score = actor.compatibility_score(process).await;
            
            // If actor has SHACL shape, validate process against it
            if let Some(shape) = actor.shacl_shape() {
                let process_jsonld = serde_json::to_value(process)
                    .map_err(|e| crate::KotobaOsError::Other(anyhow::anyhow!("Failed to serialize process: {}", e)))?;
                
                // Validate process against actor's shape
                if let Ok(validation_result) = kotoba_owl_reasoner::validate_process_shape(&process_jsonld, shape).await {
                    if validation_result.valid {
                        // Boost score if validation passes
                        scored_actors.push((score + 0.2, Arc::clone(actor)));
                        continue;
                    }
                }
            }
            
            scored_actors.push((score, Arc::clone(actor)));
        }

        // Sort by score (highest first)
        scored_actors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Select actor with highest score
        if let Some((score, actor)) = scored_actors.first() {
            if *score > 0.0 {
                info!("[Mediator] Selected actor \"{}\" with SHACL semantic score {:.2} for process \"{}\"",
                      actor.id(),
                      score,
                      process.label.as_deref().unwrap_or(&process.id));
                return Ok(Arc::clone(actor));
            }
        }

        // Fallback to capability matching
        self.select_by_capability(process).await
    }

    /// Get all registered actor IDs
    pub fn actor_ids(&self) -> Vec<String> {
        self.actors.keys().cloned().collect()
    }

    /// Check if an actor is registered
    pub fn has_actor(&self, id: &str) -> bool {
        self.actors.contains_key(id)
    }

    /// Get selection strategy
    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }
}

impl Default for Mediator {
    fn default() -> Self {
        Self::new()
    }
}
