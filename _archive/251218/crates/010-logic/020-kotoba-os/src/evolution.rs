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
        use kotoba_owl_reasoner::execute_sparql;
        
        // Extract patterns using SPARQL queries
        // Pattern 1: Frequently co-occurring processes
        let cooccurrence_query = r#"
            PREFIX kotoba: <https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab>
            SELECT ?process1 ?process2 (COUNT(*) as ?count)
            WHERE {
                ?event1 kotoba:wasGeneratedBy ?process1 .
                ?event2 kotoba:wasGeneratedBy ?process2 .
                ?event1 kotoba:endedAtTime ?time1 .
                ?event2 kotoba:endedAtTime ?time2 .
                FILTER(?process1 != ?process2)
                FILTER(ABS(?time1 - ?time2) < 3600)  # Within 1 hour
            }
            GROUP BY ?process1 ?process2
            HAVING (COUNT(*) > 3)
            ORDER BY DESC(?count)
            LIMIT 10
        "#;
        
        let mut patterns = Vec::new();
        
        // Execute pattern queries
        if let Ok(cooccurrence_result) = execute_sparql(cooccurrence_query, inferred_jsonld).await {
            if let Some(graph) = cooccurrence_result.get("@graph").and_then(|g| g.as_array()) {
                for pattern in graph {
                    patterns.push(json!({
                        "@type": "kotoba:CooccurrencePattern",
                        "kotoba:process1": pattern.get("process1"),
                        "kotoba:process2": pattern.get("process2"),
                        "kotoba:frequency": pattern.get("count"),
                    }));
                }
            }
        }
        
        // Pattern 2: Actor performance patterns
        let actor_perf_query = r#"
            PREFIX kotoba: <https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab>
            SELECT ?actor ?process (AVG(?time) as ?avgTime) (COUNT(*) as ?count)
            WHERE {
                ?event kotoba:wasGeneratedBy ?process .
                ?event kotoba:wasAssociatedWith ?actor .
                ?event kotoba:executionTime ?time .
            }
            GROUP BY ?actor ?process
            HAVING (COUNT(*) > 2)
            ORDER BY ?avgTime
        "#;
        
        if let Ok(perf_result) = execute_sparql(actor_perf_query, inferred_jsonld).await {
            if let Some(graph) = perf_result.get("@graph").and_then(|g| g.as_array()) {
                for pattern in graph {
                    patterns.push(json!({
                        "@type": "kotoba:PerformancePattern",
                        "kotoba:actor": pattern.get("actor"),
                        "kotoba:process": pattern.get("process"),
                        "kotoba:avgExecutionTime": pattern.get("avgTime"),
                        "kotoba:executionCount": pattern.get("count"),
                    }));
                }
            }
        }
        
        if patterns.is_empty() {
            None
        } else {
            Some(patterns)
        }
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
        &mut self,
        story: &Story,
    ) -> Result<Story> {
        use kotoba_owl_reasoner::{default_process_shape, validate_process_shape};
        use serde_json::json;
        
        // Convert story to JSON-LD
        let story_jsonld = serde_json::to_value(story)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize story: {}", e)))?;
        
        // Get processes from story
        let processes = if let Some(graph) = story_jsonld.get("@graph").and_then(|g| g.as_array()) {
            graph.iter()
                .filter_map(|node| {
                    if let Some(type_val) = node.get("@type").and_then(|t| t.as_str()) {
                        if type_val.contains("Process") {
                            Some(node.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        
        let mut refined_processes = Vec::new();
        
        // Refine each process shape based on metrics
        for process_jsonld in processes {
            if let Some(process_id) = process_jsonld.get("@id").and_then(|id| id.as_str()) {
                if let Some(metrics) = self.metrics.get(process_id) {
                    // Get current shape
                    let mut shape = default_process_shape();
                    
                    // Refine shape based on metrics
                    if metrics.success_rate < 0.5 {
                        // Low success rate - make constraints stricter
                        if let Some(properties) = shape.get_mut("sh:property").and_then(|p| p.as_array_mut()) {
                            for prop in properties {
                                if let Some(path) = prop.get("sh:path").and_then(|p| p.as_str()) {
                                    if path.contains("performedBy") {
                                        // Ensure performedBy is required
                                        prop.as_object_mut().unwrap().insert(
                                            "sh:minCount".to_string(),
                                            json!(1)
                                        );
                                    }
                                }
                            }
                        }
                    }
                    
                    // If preferred actor exists, add constraint
                    if let Some(preferred_actor) = &metrics.preferred_actor {
                        if let Some(properties) = shape.get_mut("sh:property").and_then(|p| p.as_array_mut()) {
                            properties.push(json!({
                                "sh:path": "kotoba:preferredActor",
                                "sh:hasValue": preferred_actor,
                                "sh:minCount": 0,
                                "sh:maxCount": 1
                            }));
                        }
                    }
                    
                    // Validate refined shape
                    let validation_result = validate_process_shape(&process_jsonld, &shape).await
                        .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Shape validation failed: {}", e)))?;
                    
                    if validation_result.valid {
                        // Record refinement
                        self.record_evolution(json!({
                            "@type": "kotoba:ShapeRefinement",
                            "kotoba:processId": process_id,
                            "kotoba:refinedShape": shape,
                            "kotoba:reason": format!("Success rate: {:.2}%, Preferred actor: {:?}", 
                                metrics.success_rate * 100.0, metrics.preferred_actor),
                        }));
                        
                        refined_processes.push(process_jsonld);
                    } else {
                        // Keep original if refinement invalid
                        warn!("[EvolutionEngine] Shape refinement failed for process {}, keeping original", process_id);
                        refined_processes.push(process_jsonld);
                    }
                } else {
                    // No metrics available, keep original
                    refined_processes.push(process_jsonld);
                }
            } else {
                refined_processes.push(process_jsonld);
            }
        }
        
        // Reconstruct story with refined processes
        let mut refined_story_jsonld = story_jsonld.clone();
        if let Some(graph) = refined_story_jsonld.get_mut("@graph").and_then(|g| g.as_array_mut()) {
            // Replace processes with refined ones
            for (i, node) in graph.iter_mut().enumerate() {
                if let Some(type_val) = node.get("@type").and_then(|t| t.as_str()) {
                    if type_val.contains("Process") {
                        if let Some(id) = node.get("@id").and_then(|id| id.as_str()) {
                            if let Some(refined) = refined_processes.iter().find(|p| {
                                p.get("@id").and_then(|pid| pid.as_str()) == Some(id)
                            }) {
                                graph[i] = refined.clone();
                            }
                        }
                    }
                }
            }
        }
        
        // Convert back to Story
        let refined_story = Story::from_value(refined_story_jsonld)
            .map_err(|e| KotobaOsError::StoryValidation(e.to_string()))?;
        
        info!("[EvolutionEngine] Refined {} processes", refined_processes.len());
        Ok(refined_story)
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

