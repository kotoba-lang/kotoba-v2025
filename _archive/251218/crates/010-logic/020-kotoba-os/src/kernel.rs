//! Kernel implementation for KotobaOS
//!
//! The central orchestrator that manages process execution lifecycle.

use crate::actor::{ActorTrait, DefaultActor};
use crate::error::{ErrorContext, ErrorEscalator, RetryConfig, RetryExecutor};
use crate::mediator::Mediator;
use crate::process_handler::ProcessHandler;
use crate::provenance::Provenance;
use crate::types::{Process, Story};
use crate::{Result, KotobaOsError};
use kotoba_storage::StorageEngine;
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

    /// Retry executor for error handling
    retry_executor: RetryExecutor,
    /// Error escalator for error management
    error_escalator: ErrorEscalator,
    /// GraphStream engine for real-time processing (optional)
    graph_stream: Option<Arc<GraphStream>>,

    /// OWL reasoning engine (optional)
    #[cfg(feature = "reasoning")]
    reasoning_engine: Option<kotoba_owl_reasoner::ReasoningEngine>,
}

impl Kernel {
    /// Create a new kernel with a story and storage
    pub fn new(story: Value, storage: Arc<dyn StorageEngine>) -> Result<Self> {
        let story: Story = Story::from_value(story)
            .map_err(|e| KotobaOsError::StoryValidation(e.to_string()))?;

        Ok(Self {
            mediator: Mediator::new(),
            provenance: Provenance::new(storage),
            story,
            on_process_start: None,
            on_process_end: None,
            retry_executor: RetryExecutor::default(),
            error_escalator: ErrorEscalator::default(),
            graph_stream: None,
            #[cfg(feature = "reasoning")]
            reasoning_engine: None,
        })
    }

    /// Create a new kernel with OWL reasoning enabled
    #[cfg(feature = "reasoning")]
    pub fn with_reasoning(
        story: Value,
        storage: Arc<dyn StorageEngine>,
        reasoning_level: kotoba_owl_reasoner::ReasoningLevel,
    ) -> Result<Self> {
        let story: Story = Story::from_value(story)
            .map_err(|e| KotobaOsError::StoryValidation(e.to_string()))?;

        let reasoning_engine = kotoba_owl_reasoner::ReasoningEngine::new(reasoning_level)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to create reasoning engine: {}", e)))?;

        Ok(Self {
            mediator: Mediator::new(),
            provenance: Provenance::new(storage),
            story,
            on_process_start: None,
            on_process_end: None,
            retry_executor: RetryExecutor::default(),
            error_escalator: ErrorEscalator::default(),
            graph_stream: None,
            reasoning_engine: Some(reasoning_engine),
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

        // Select actor via mediator (with retry)
        let mediator = &self.mediator;
        let retry_executor = &self.retry_executor;
        let error_escalator = &self.error_escalator;
        let process_clone = process.clone();
        let process_id = process.id.clone();

        let actor = match retry_executor
            .execute(
                || {
                    let process = process_clone.clone();
                    Box::pin(async move {
                        mediator.select_actor(&process).await.map_err(|e| {
                            format!("{}", e)
                        })
                    })
                },
                &format!("select_actor({})", process_id),
            )
            .await
        {
            Ok(actor) => actor,
            Err(e) => {
                let error_ctx = ErrorContext::from_error(
                    &KotobaOsError::ActorSelection(e.message.clone()),
                    Some(process_id.clone()),
                );
                let escalation = error_escalator.escalate(&error_ctx);
                error_escalator.handle_escalation(&error_ctx, escalation);
                return Err(KotobaOsError::ActorSelection(format!(
                    "Failed to select actor after retries: {}",
                    e.message
                )));
            }
        };

        // Execute action via actor (with retry)
        let process_clone2 = process.clone();
        let result = match retry_executor
            .execute(
                || {
                    let actor = actor.clone();
                    let process = process_clone2.clone();
                    Box::pin(async move {
                        actor.perform(&process).await.map_err(|e| {
                            format!("{}", e)
                        })
                    })
                },
                &format!("perform({})", process_id),
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let error_ctx = ErrorContext::from_error(
                    &KotobaOsError::ProcessExecution(e.message.clone()),
                    Some(process_id.clone()),
                );
                let escalation = error_escalator.escalate(&error_ctx);
                error_escalator.handle_escalation(&error_ctx, escalation);
                return Err(KotobaOsError::ProcessExecution(format!(
                    "Failed to execute process after retries: {}",
                    e.message
                )));
            }
        };

        // Record provenance (with retry)
        // Note: Provenance recording is already resilient (includes storage retry),
        // so we use a simplified retry approach here
        let process_clone3 = process.clone();
        let result_clone = result.clone();
        let actor_clone = actor.clone();
        
        // Retry provenance recording with exponential backoff
        let mut last_error: Option<KotobaOsError> = None;
        let config = &retry_executor.config;
        for attempt in 0..=config.max_retries {
            match self.provenance.record(&process_clone3, &actor_clone, &result_clone).await {
                Ok(_) => {
                    if attempt > 0 {
                        info!(
                            "[Kernel] Provenance recording succeeded after {} retries for process {}",
                            attempt, process_id
                        );
                    }
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < config.max_retries {
                        let delay_secs = (config.initial_delay_secs as f64
                            * config.backoff_multiplier.powi(attempt as i32))
                            .min(config.max_delay_secs as f64) as u64;
                        let delay = Duration::from_secs(delay_secs);
                        warn!(
                            "[Kernel] Provenance recording failed (attempt {}/{}), retrying in {:?}: {}",
                            attempt + 1,
                            config.max_retries,
                            delay,
                            e
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        if let Some(e) = last_error {
            let error_ctx = ErrorContext::from_error(&e, Some(process_id.clone()));
            let escalation = error_escalator.escalate(&error_ctx);
            error_escalator.handle_escalation(&error_ctx, escalation);
            return Err(KotobaOsError::ProvenanceError(format!(
                "Failed to record provenance after {} retries: {}",
                config.max_retries, e
            )));
        }

        // Publish to GraphStream if enabled
        if let Some(ref stream) = self.graph_stream {
            let events = self.provenance.events();
            if let Some(event) = events.last() {
                if let Err(e) = stream.publish(event).await {
                    warn!("[Kernel] Failed to publish event to GraphStream: {}", e);
                }
            }
        }

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

        // Load existing provenance from storage
        if let Err(e) = self.provenance.load_from_storage().await {
            warn!("[Kernel] Failed to load provenance from storage: {}", e);
            // Continue anyway - this is not fatal
        }

        // Create process handler
        let handler = ProcessHandler::new(self.story.clone());

        // Get ordered process chain
        let process_chain = handler.get_process_chain();

        // Execute each process in the chain
        for process in process_chain {
            self.run_process(&process).await?;
        }

        // Perform evolution analysis if enabled
        #[cfg(feature = "reasoning")]
        if let Some(ref mut evolution) = self.evolution_engine {
            if let Err(e) = evolution.analyze_provenance(&self.provenance).await {
                warn!("[Kernel] Evolution analysis failed: {}", e);
            } else {
                // Refine shapes based on discovered patterns
                match evolution.refine_shapes(&self.story).await {
                    Ok(refined_story) => {
                        let pattern_count = evolution.evolution_history_jsonld()
                            .get("@graph")
                            .and_then(|g| g.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        info!("[Kernel] Shape refinement completed, {} patterns discovered", pattern_count);
                        // Note: Refined story could be used in next iteration
                        // For now, we just log the refinement
                    }
                    Err(e) => {
                        warn!("[Kernel] Shape refinement failed: {}", e);
                    }
                }
            }
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

    /// Get evolution engine
    #[cfg(feature = "reasoning")]
    pub fn evolution_engine(&self) -> Option<&crate::evolution::EvolutionEngine> {
        self.evolution_engine.as_ref()
    }

    /// Get evolution engine (mutable)
    #[cfg(feature = "reasoning")]
    pub fn evolution_engine_mut(&mut self) -> Option<&mut crate::evolution::EvolutionEngine> {
        self.evolution_engine.as_mut()
    }

    /// Enable evolution with strategy
    #[cfg(feature = "reasoning")]
    pub fn enable_evolution(&mut self, strategy: crate::evolution::EvolutionStrategy) {
        if let Some(ref mut evolution) = self.evolution_engine {
            evolution.set_strategy(strategy);
        } else {
            self.evolution_engine = Some(crate::evolution::EvolutionEngine::with_strategy(strategy));
        }
    }

    /// Get evolution history as JSON-LD
    #[cfg(feature = "reasoning")]
    pub fn evolution_history_jsonld(&self) -> Option<Value> {
        self.evolution_engine.as_ref().map(|e| e.evolution_history_jsonld())
    }

    /// Enable GraphStream for real-time processing
    pub fn enable_graph_stream(&mut self, stream: GraphStream) {
        self.graph_stream = Some(Arc::new(stream));
        info!("[Kernel] GraphStream enabled");
    }

    /// Get GraphStream instance (if enabled)
    pub fn graph_stream(&self) -> Option<Arc<GraphStream>> {
        self.graph_stream.clone()
    }
}
