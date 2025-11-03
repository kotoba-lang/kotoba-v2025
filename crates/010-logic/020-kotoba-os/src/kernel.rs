//! Kernel implementation for KotobaOS
//!
//! The central orchestrator that manages process execution lifecycle.

use crate::actor::{ActorTrait, DefaultActor};
use crate::mediator::Mediator;
use crate::process_handler::ProcessHandler;
use crate::provenance::Provenance;
use crate::types::{Process, Story};
use crate::{Result, KotobaOsError};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

/// Kernel orchestrates the entire process execution lifecycle
pub struct Kernel {
    /// Mediator for actor selection
    pub mediator: Mediator,

    /// Provenance service for recording execution history
    pub provenance: Provenance,

    /// Story containing the process network
    story: Story,

    /// Optional callback when a process starts
    pub on_process_start: Option<Box<dyn Fn(&Process) + Send + Sync>>,

    /// Optional callback when a process ends
    pub on_process_end: Option<Box<dyn Fn(&Process) + Send + Sync>>,

    /// OWL reasoning engine (optional)
    #[cfg(feature = "reasoning")]
    reasoning_engine: Option<kotoba_owl_reasoner::ReasoningEngine>,
}

impl Kernel {
    /// Create a new kernel with a story
    pub fn new(story: Value) -> Result<Self> {
        let story: Story = Story::from_value(story)
            .map_err(|e| KotobaOsError::StoryValidation(e.to_string()))?;

        Ok(Self {
            mediator: Mediator::new(),
            provenance: Provenance::new(),
            story,
            on_process_start: None,
            on_process_end: None,
            #[cfg(feature = "reasoning")]
            reasoning_engine: None,
            #[cfg(feature = "reasoning")]
            shacl_validator: None,
        })
    }

    /// Create a new kernel with OWL reasoning enabled
    #[cfg(feature = "reasoning")]
    pub fn with_reasoning(
        story: Value,
        reasoning_level: kotoba_owl_reasoner::ReasoningLevel,
    ) -> Result<Self> {
        let story: Story = Story::from_value(story)
            .map_err(|e| KotobaOsError::StoryValidation(e.to_string()))?;

        let reasoning_engine = kotoba_owl_reasoner::ReasoningEngine::new(reasoning_level)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to create reasoning engine: {}", e)))?;

        Ok(Self {
            mediator: Mediator::new(),
            provenance: Provenance::new(),
            story,
            on_process_start: None,
            on_process_end: None,
            reasoning_engine: Some(reasoning_engine),
            shacl_validator: Some(crate::ShaclValidator::new()),
        })
    }

    /// Register an actor with the kernel
    pub fn register_actor<A: ActorTrait + 'static>(&mut self, actor: A) {
        self.mediator.register_actor(actor);
    }

    /// Register a default actor (convenience method)
    pub fn register_default_actor(&mut self, id: impl Into<String>, capability: impl Into<String>) {
        self.register_actor(DefaultActor::new(id, capability));
    }

    /// Run a single process
    /// 
    /// This is the core execution logic:
    /// 1. Select Actor via Mediator
    /// 2. Execute action via Actor
    /// 3. Record provenance
    pub async fn run_process(&mut self, process: &Process) -> Result<()> {
        info!("[Kernel] Running process: {}", process.id);

        // Call on_process_start callback
        if let Some(callback) = &self.on_process_start {
            callback(process);
        }

        // Perform OWL reasoning if enabled
        #[cfg(feature = "reasoning")]
        if let Some(ref mut engine) = self.reasoning_engine {
            // Convert process to JSON-LD for reasoning
            let process_jsonld = serde_json::to_value(process)
                .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize process: {}", e)))?;
            
            // Load process into reasoning engine
            if let Err(e) = engine.load_ontology_from_jsonld(process_jsonld).await {
                warn!("[Kernel] OWL reasoning failed for process {}: {}", process.id, e);
            } else {
                // Perform reasoning
                if let Ok(reasoning_result) = engine.reason().await {
                    info!("[Kernel] OWL reasoning inferred {} triples for process {}", 
                          reasoning_result.inferred_triples.len(), process.id);
                }
            }
        }

        // Perform SHACL validation if enabled
        #[cfg(feature = "reasoning")]
        if let Some(ref validator) = self.shacl_validator {
            if let Err(e) = validator.validate_process(process).await {
                warn!("[Kernel] SHACL validation failed for process {}: {}", process.id, e);
                // In strict mode, this would return an error
                // For now, we just log the warning
            }
        }

        // Select actor via mediator
        let actor = self.mediator.select_actor(process).await?;

        // Execute action via actor
        let result = actor.perform(process).await?;

        // Record provenance
        self.provenance
            .record(process, &actor, &result)
            .await?;

        // Call on_process_end callback
        if let Some(callback) = &self.on_process_end {
            callback(process);
        }

        info!("[Kernel] Process completed: {}", process.id);
        Ok(())
    }

    /// Start the orchestration of the entire story
    pub async fn start(&mut self) -> Result<()> {
        info!("[Kernel] Booting up...");

        // Create process handler
        let handler = ProcessHandler::new(self.story.clone());

        // Get ordered process chain
        let process_chain = handler.get_process_chain();

        // Execute each process in the chain
        for process in process_chain {
            self.run_process(&process).await?;
        }

        info!("[Kernel] Shutdown.");
        Ok(())
    }

    /// Get the story
    pub fn story(&self) -> &Story {
        &self.story
    }

    /// Get provenance events as JSON-LD
    pub fn provenance_jsonld(&self) -> Value {
        self.provenance.to_jsonld()
    }

    /// Get inferred triples from OWL reasoning (if enabled)
    #[cfg(feature = "reasoning")]
    pub async fn get_inferred_triples(&self) -> Option<Value> {
        if let Some(ref engine) = self.reasoning_engine {
            engine.inferred_triples_as_jsonld().await.ok()
        } else {
            None
        }
    }

    /// Set SHACL validator
    #[cfg(feature = "reasoning")]
    pub fn set_shacl_validator(&mut self, validator: crate::ShaclValidator) {
        self.shacl_validator = Some(validator);
    }

    /// Enable strict SHACL validation
    #[cfg(feature = "reasoning")]
    pub fn enable_strict_validation(&mut self) {
        self.shacl_validator = Some(crate::ShaclValidator::strict());
    }
}
