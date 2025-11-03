//! JSON-LD reasoning integration
//!
//! Provides integration with OWL reasoning for JSON-LD documents.

use crate::{parse_jsonld_to_value, JsonLdDocument};
use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};
use serde_json::Value;
use anyhow::Result;

/// Parse JSON-LD with automatic reasoning
/// 
/// This function parses JSON-LD and optionally performs OWL reasoning
/// to infer additional triples.
pub async fn parse_jsonld_with_reasoning(
    jsonld_str: &str,
    reasoning_level: Option<ReasoningLevel>,
) -> Result<Value> {
    // Parse JSON-LD
    let jsonld_value = parse_jsonld_to_value(jsonld_str)?;
    
    // If no reasoning requested, return as-is
    if reasoning_level.is_none() {
        return Ok(jsonld_value);
    }
    
    // Create reasoning engine
    let mut engine = ReasoningEngine::new(reasoning_level.unwrap())
        .map_err(|e| anyhow::anyhow!("Failed to create reasoning engine: {}", e))?;
    
    // Load ontology
    engine.load_ontology_from_jsonld(jsonld_value.clone()).await
        .map_err(|e| anyhow::anyhow!("Failed to load ontology: {}", e))?;
    
    // Perform reasoning
    let reasoning_result = engine.reason().await
        .map_err(|e| anyhow::anyhow!("Reasoning failed: {}", e))?;
    
    // Get inferred triples as JSON-LD
    let inferred_jsonld = engine.inferred_triples_as_jsonld().await
        .map_err(|e| anyhow::anyhow!("Failed to get inferred triples: {}", e))?;
    
    // Merge original and inferred
    merge_jsonld_documents(&jsonld_value, &inferred_jsonld)
}

/// Expand JSON-LD with OWL reasoning
pub async fn expand_jsonld_with_owl(jsonld: &Value, level: ReasoningLevel) -> Result<Value> {
    let mut engine = ReasoningEngine::new(level)
        .map_err(|e| anyhow::anyhow!("Failed to create reasoning engine: {}", e))?;
    
    engine.load_ontology_from_jsonld(jsonld.clone()).await
        .map_err(|e| anyhow::anyhow!("Failed to load ontology: {}", e))?;
    
    let _reasoning_result = engine.reason().await
        .map_err(|e| anyhow::anyhow!("Reasoning failed: {}", e))?;
    
    engine.inferred_triples_as_jsonld().await
        .map_err(|e| anyhow::anyhow!("Failed to get inferred triples: {}", e))
}

/// Validate JSON-LD with SHACL
pub async fn validate_jsonld_with_shacl(data: &Value, shape: &Value) -> Result<bool> {
    use kotoba_owl_reasoner::shacl::validate_shacl;
    
    let validation_result = validate_shacl(data, shape).await
        .map_err(|e| anyhow::anyhow!("SHACL validation failed: {}", e))?;
    
    Ok(validation_result.valid)
}

/// Merge two JSON-LD documents
fn merge_jsonld_documents(doc1: &Value, doc2: &Value) -> Result<Value> {
    let mut merged = doc1.clone();
    
    // Merge @graph arrays
    if let Some(graph1) = doc1.get("@graph").and_then(|g| g.as_array()) {
        if let Some(graph2) = doc2.get("@graph").and_then(|g| g.as_array()) {
            let mut combined = graph1.clone();
            combined.extend(graph2.iter().cloned());
            
            if let Some(merged_obj) = merged.as_object_mut() {
                merged_obj.insert("@graph".to_string(), Value::Array(combined));
            }
        }
    }
    
    Ok(merged)
}

