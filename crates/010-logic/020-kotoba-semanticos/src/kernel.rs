//! Kernel implementation for SemanticOS
//!
//! The central orchestrator that manages process execution lifecycle.

use crate::actor::{ActorTrait, DefaultActor};
use crate::mediator::Mediator;
use crate::process_handler::ProcessHandler;
use crate::provenance::Provenance;
use crate::types::{Process, Story};
use crate::{Result, SemanticOsError};
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
}

impl Kernel {
    /// Create a new kernel with a story
    pub fn new(story: Value) -> Result<Self> {
        let story: Story = Story::from_value(story)
            .map_err(|e| SemanticOsError::StoryValidation(e.to_string()))?;

        Ok(Self {
            mediator: Mediator::new(),
            provenance: Provenance::new(),
            story,
            on_process_start: None,
            on_process_end: None,
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
}

