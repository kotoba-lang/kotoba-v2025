//! # Kotoba OWL Reasoner
//!
//! OWL reasoning engine integration for Kotoba using fukurow.
//! Provides RDFS, OWL Lite, and OWL DL reasoning capabilities with JSON-LD support.
//!
//! ## Features
//!
//! - **RDFS Reasoning**: Transitive closure for subClassOf and subPropertyOf
//! - **OWL Lite Reasoning**: Tableau algorithm for class hierarchy inference
//! - **OWL DL Reasoning**: Complete OWL DL reasoning with all constructors
//! - **SHACL Validation**: Shape constraint validation
//! - **SPARQL Queries**: SPARQL 1.1 query execution
//! - **JSON-LD Integration**: Seamless JSON-LD input/output
//!
//! ## Example
//!
//! ```rust,no_run
//! use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create reasoning engine
//! let mut engine = ReasoningEngine::new(ReasoningLevel::OwlDl)?;
//!
//! // Load ontology from JSON-LD
//! let ontology_json = json!({
//!     "@context": {
//!         "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
//!         "owl": "http://www.w3.org/2002/07/owl#"
//!     },
//!     "@graph": [
//!         {
//!             "@id": "ex:Person",
//!             "@type": "owl:Class"
//!         },
//!         {
//!             "@id": "ex:Student",
//!             "@type": "owl:Class",
//!             "rdfs:subClassOf": "ex:Person"
//!         }
//!     ]
//! });
//!
//! engine.load_ontology_from_jsonld(ontology_json).await?;
//!
//! // Perform reasoning
//! let inferred = engine.reason().await?;
//!
//! // Get inferred triples as JSON-LD
//! let inferred_jsonld = engine.inferred_triples_as_jsonld().await?;
//! # Ok(())
//! # }
//! ```

pub mod reasoner;
pub mod rdfs;
pub mod owl_lite;
pub mod owl_dl;
pub mod shacl;
pub mod sparql;
pub mod fukurow_binding;

pub use reasoner::{ReasoningEngine, ReasoningLevel, ReasoningResult};
pub use fukurow_binding::FukurowStore;
pub use shacl::{
    ShaclValidationResult, validate_shacl, validate_process_shape,
    validate_resource_shape, validate_performer_shape,
    default_process_shape, default_resource_shape, default_performer_shape,
};
pub use sparql::{execute_sparql, compile_shape_to_sparql};

/// Error types for OWL reasoning operations
#[derive(Debug, thiserror::Error)]
pub enum OwlReasonerError {
    #[error("JSON-LD parsing error: {0}")]
    JsonLdParse(#[from] serde_json::Error),

    #[error("RDF store error: {0}")]
    StoreError(String),

    #[error("Reasoning error: {0}")]
    ReasoningError(String),

    #[error("SHACL validation error: {0}")]
    ShaclError(String),

    #[error("SPARQL query error: {0}")]
    SparqlError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, OwlReasonerError>;

