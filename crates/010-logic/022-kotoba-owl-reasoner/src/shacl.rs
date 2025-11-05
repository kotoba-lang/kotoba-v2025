//! SHACL validation implementation
//!
//! Provides SHACL shape constraint validation using fukurow-shacl.

use crate::fukurow_binding::FukurowStore;
use crate::Result;
use fukurow_shacl::loader::{DefaultShaclLoader, ShaclLoader};
use fukurow_shacl::validator::{DefaultShaclValidator, ShaclValidator, ValidationConfig, ValidationMode};
use serde_json::{json, Value};
use tracing::{info, warn};

/// SHACL validation result
#[derive(Debug, Clone)]
pub struct ShaclValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation report as JSON-LD
    pub report: Value,
}

/// Validate data against SHACL shape
pub async fn validate_shacl(data: &Value, shape: &Value) -> Result<ShaclValidationResult> {
    // 1. Convert JSON-LD data to RdfStore
    let mut data_store = FukurowStore::new();
    data_store.load_from_jsonld(data.clone()).await?;
    
    // 2. Convert JSON-LD shape to RdfStore and load ShapesGraph
    let mut shape_store = FukurowStore::new();
    shape_store.load_from_jsonld(shape.clone()).await?;
    
    let loader = DefaultShaclLoader;
    let rdf_store = shape_store.store_guard().await;
    let shapes_graph = loader.load_from_store(&rdf_store)
        .map_err(|e| crate::OwlReasonerError::Other(anyhow::anyhow!("Failed to load SHACL shapes: {}", e)))?;
    
    drop(rdf_store); // Release the guard
    
    // 3. Run validation
    let validator = DefaultShaclValidator;
    let config = ValidationConfig {
        mode: ValidationMode::Warn, // Warn mode to collect all violations
        report_jsonld: true,
    };
    
    let data_rdf_store = data_store.store_guard().await;
    let validation_report = validator.validate_graph(&shapes_graph, &data_rdf_store, &config)
        .map_err(|e| crate::OwlReasonerError::Other(anyhow::anyhow!("SHACL validation failed: {}", e)))?;
    
    drop(data_rdf_store); // Release the guard
    
    // Extract errors from validation report
    let errors: Vec<String> = validation_report.results
        .iter()
        .filter_map(|r| r.message.clone())
        .collect();
    
    // Convert report to JSON-LD
    let report_jsonld = validation_report.to_jsonld()
        .map_err(|e| crate::OwlReasonerError::Other(anyhow::anyhow!("Failed to serialize validation report: {}", e)))?;
    
    Ok(ShaclValidationResult {
        valid: validation_report.conforms,
        errors,
        report: report_jsonld,
    })
}

/// Validate Process, Resource, or Performer against SHACL shape
pub async fn validate_process_shape(
    process_jsonld: &Value,
    shape_jsonld: &Value,
) -> Result<ShaclValidationResult> {
    // Use the general validate_shacl function
    validate_shacl(process_jsonld, shape_jsonld).await
}

/// Validate Resource against SHACL shape
pub async fn validate_resource_shape(
    resource_jsonld: &Value,
    shape_jsonld: &Value,
) -> Result<ShaclValidationResult> {
    validate_process_shape(resource_jsonld, shape_jsonld).await
}

/// Validate Performer against SHACL shape
pub async fn validate_performer_shape(
    performer_jsonld: &Value,
    shape_jsonld: &Value,
) -> Result<ShaclValidationResult> {
    validate_process_shape(performer_jsonld, shape_jsonld).await
}

/// Create a default Process shape for validation
pub fn default_process_shape() -> Value {
    json!({
        "@context": {
            "sh": "http://www.w3.org/ns/shacl#",
            "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
        },
        "@type": "sh:NodeShape",
        "sh:targetClass": "kotoba:Process",
        "sh:property": [
            {
                "sh:path": "kotoba:performedBy",
                "sh:minCount": 1,
                "sh:maxCount": 1,
                "sh:nodeKind": "sh:IRI"
            },
            {
                "sh:path": "kotoba:label",
                "sh:datatype": "xsd:string",
                "sh:minCount": 0,
                "sh:maxCount": 1
            }
        ]
    })
}

/// Create a default Resource shape for validation
pub fn default_resource_shape() -> Value {
    json!({
        "@context": {
            "sh": "http://www.w3.org/ns/shacl#",
            "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
        },
        "@type": "sh:NodeShape",
        "sh:targetClass": "kotoba:Resource",
        "sh:property": [
            {
                "sh:path": "kotoba:label",
                "sh:datatype": "xsd:string",
                "sh:minCount": 0,
                "sh:maxCount": 1
            }
        ]
    })
}

/// Create a default Performer shape for validation
pub fn default_performer_shape() -> Value {
    json!({
        "@context": {
            "sh": "http://www.w3.org/ns/shacl#",
            "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
        },
        "@type": "sh:NodeShape",
        "sh:targetClass": "kotoba:Performer",
        "sh:property": [
            {
                "sh:path": "kotoba:capability",
                "sh:nodeKind": "sh:IRI",
                "sh:minCount": 0,
                "sh:maxCount": 1
            },
            {
                "sh:path": "kotoba:label",
                "sh:datatype": "xsd:string",
                "sh:minCount": 0,
                "sh:maxCount": 1
            }
        ]
    })
}
