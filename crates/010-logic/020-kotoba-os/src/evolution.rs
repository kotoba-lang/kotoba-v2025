//! Self-evolution mechanism for KotobaOS
//!
//! Implements the Semantic Design Loop:
//! Shape → Process → Provenance → Pattern Discovery → Shape Refinement
//!
//! Uses OWL inference to discover optimization patterns and automatically
//! refines SHACL shapes based on execution history.

use crate::provenance::Provenance;
use crate::types::{Process, Story};
use crate::{KotobaOsError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn};

/// Performance metrics for a process
#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    /// Process ID
    pub process_id: String,
    /// Number of executions
    pub execution_count: u64,
    /// Average execution time (seconds)
    pub avg_execution_time: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Total errors
    pub error_count: u64,
    /// Actor that performed this process most often
    pub preferred_actor: Option<String>,
}

/// Evolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStrategy {
    /// No evolution
    None,
    /// Pattern-based evolution (OWL inference)
    PatternBased,
    /// Performance-based evolution (metrics-driven)
    PerformanceBased,
    /// Hybrid (pattern + performance)
    Hybrid,
}

/// Evolution engine for self-improvement
pub struct EvolutionEngine {
    /// Evolution strategy
    strategy: EvolutionStrategy,
    /// Performance metrics by process ID
    metrics: HashMap<String, ProcessMetrics>,
    /// Evolution history (JSON-LD format)
    evolution_history: Vec<Value>,
}

impl EvolutionEngine {
    /// Create a new evolution engine
    pub fn new() -> Self {
        Self {
            strategy: EvolutionStrategy::None,
            metrics: HashMap::new(),
            evolution_history: Vec::new(),
        }
    }

    /// Create with strategy
    pub fn with_strategy(strategy: EvolutionStrategy) -> Self {
        Self {
            strategy,
            metrics: HashMap::new(),
            evolution_history: Vec::new(),
        }
    }

    /// Set evolution strategy
    pub fn set_strategy(&mut self, strategy: EvolutionStrategy) {
        self.strategy = strategy;
    }

    /// Analyze provenance and discover patterns
    #[cfg(feature = "reasoning")]
    pub async fn analyze_provenance(
        &mut self,
        provenance: &Provenance,
    ) -> Result<Vec<Value>> {
        if self.strategy == EvolutionStrategy::None {
            return Ok(Vec::new());
        }

        let provenance_jsonld = provenance.to_jsonld();

        // Use OWL inference to discover patterns
        if matches!(
            self.strategy,
            EvolutionStrategy::PatternBased | EvolutionStrategy::Hybrid
        ) {
            self.discover_patterns(&provenance_jsonld).await?;
        }

        // Analyze performance metrics
        if matches!(
            self.strategy,
            EvolutionStrategy::PerformanceBased | EvolutionStrategy::Hybrid
        ) {
            self.analyze_performance(provenance).await?;
        }

        Ok(self.evolution_history.clone())
    }

    /// Discover optimization patterns using OWL inference
    #[cfg(feature = "reasoning")]
    async fn discover_patterns(&mut self, provenance_jsonld: &Value) -> Result<()> {
        use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};

        // Create reasoning engine
        let mut engine = ReasoningEngine::new(ReasoningLevel::OwlDl)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to create reasoning engine: {}", e)))?;

        // Load provenance into reasoning engine
        engine
            .load_ontology_from_jsonld(provenance_jsonld.clone())
            .await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to load provenance: {}", e)))?;

        // Perform reasoning
        let reasoning_result = engine
            .reason()
            .await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Reasoning failed: {}", e)))?;

        // Get inferred triples
        let inferred = engine
            .inferred_triples_as_jsonld()
            .await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to get inferred triples: {}", e)))?;

        // Extract patterns from inferred triples
        if let Some(patterns) = self.extract_patterns(&inferred).await {
            info!("[EvolutionEngine] Discovered {} patterns", patterns.len());
            self.evolution_history.extend(patterns);
        }

        Ok(())
    }

    /// Extract optimization patterns from inferred triples
    #[cfg(feature = "reasoning")]
    async fn extract_patterns(&self, inferred_jsonld: &Value) -> Option<Vec<Value>> {
        // TODO: Implement pattern extraction logic
        // For now, return placeholder
        Some(Vec::new())
    }

    /// Analyze performance metrics from provenance
    async fn analyze_performance(&mut self, provenance: &Provenance) -> Result<()> {
        let events = provenance.events();

        // Group events by process ID
        let mut process_stats: HashMap<String, Vec<&crate::types::ProvenanceEvent>> =
            HashMap::new();

        for event in events {
            process_stats
                .entry(event.was_generated_by.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }

        // Calculate metrics for each process
        for (process_id, events) in process_stats {
            let execution_count = events.len() as u64;
            let error_count = events
                .iter()
                .filter(|e| {
                    e.additional
                        .get("kotoba:error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .count() as u64;

            let success_rate = if execution_count > 0 {
                1.0 - (error_count as f64 / execution_count as f64)
            } else {
                0.0
            };

            // Calculate average execution time (if available)
            let avg_execution_time = events
                .iter()
                .filter_map(|e| {
                    e.additional
                        .get("kotoba:executionTime")
                        .and_then(|v| v.as_f64())
                })
                .sum::<f64>()
                / execution_count as f64;

            // Find preferred actor
            let mut actor_counts: HashMap<String, u64> = HashMap::new();
            for event in events {
                if let Some(actor_id) = &event.was_associated_with {
                    *actor_counts.entry(actor_id.clone()).or_insert(0) += 1;
                }
            }
            let preferred_actor = actor_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(actor_id, _)| actor_id.clone());

            let metrics = ProcessMetrics {
                process_id: process_id.clone(),
                execution_count,
                avg_execution_time,
                success_rate,
                error_count,
                preferred_actor,
            };

            self.metrics.insert(process_id, metrics);
        }

        info!("[EvolutionEngine] Analyzed performance for {} processes", self.metrics.len());
        Ok(())
    }

    /// Refine SHACL shapes based on discovered patterns and metrics
    #[cfg(feature = "reasoning")]
    pub async fn refine_shapes(
        &self,
        story: &Story,
    ) -> Result<Story> {
        // TODO: Implement shape refinement logic
        // For now, return original story
        Ok(story.clone())
    }

    /// Get performance metrics for a process
    pub fn get_metrics(&self, process_id: &str) -> Option<&ProcessMetrics> {
        self.metrics.get(process_id)
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> &HashMap<String, ProcessMetrics> {
        &self.metrics
    }

    /// Get evolution history as JSON-LD
    pub fn evolution_history_jsonld(&self) -> Value {
        json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
            },
            "@type": "kotoba:EvolutionHistory",
            "@graph": self.evolution_history
        })
    }

    /// Record an evolution event
    pub fn record_evolution(&mut self, event: Value) {
        self.evolution_history.push(event);
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

