//! # Kotoba: Phonosemantic Digital Computing System
//!
//! A phonosemantic digital computing system where all computing, operating system,
//! datastore, and self-evolution mechanisms are represented, reasoned, and executed
//! using JSON-LD with OWL inference.
//!
//! ## Architecture Overview
//!
//! Kotoba integrates three foundational concepts:
//!
//! 1. **Phonosemantic Vocabulary System**: Systematic mapping between phonemes (sound units)
//!    and semantic meanings, enabling natural language understanding through structured
//!    vocabulary relationships.
//!
//! 2. **OWL Inference Engine**: Complete reasoning capabilities using RDFS, OWL Lite, and
//!    OWL DL inference engines (powered by [fukurow](https://github.com/com-junkawasaki/fukurow))
//!    for logical deduction and knowledge discovery.
//!
//! 3. **Semantic Execution Pattern**: A Kernel + Actor + Mediator architecture (inspired by
//!    [semanticos](https://github.com/com-junkawasaki/semanticos)) for executing process networks
//!    defined in JSON-LD with automatic actor selection and provenance tracking.
//!
//! ## Key Features
//!
//! - **JSON-LD Native**: All computing layers use JSON-LD for representation
//! - **OWL Inference**: RDFS + OWL Lite + OWL DL complete reasoning
//! - **Phonosemantic Mapping**: Bidirectional phoneme ↔ meaning conversion
//! - **Semantic Execution**: Kernel + Actor + Mediator pattern
//! - **Self-Evolution**: Semantic Design Loop for continuous improvement
//! - **Provenance Tracking**: Complete execution history in JSON-LD/PROV-O format
//! - **Process Network Graph**: Declarative configuration with automatic dependency resolution
//! - **MVCC + Merkle DAG**: Consistent distributed data management
//!
//! ## Usage
//!
//! ```rust
//! use kotoba::*;
//!
//! // Create storage adapter
//! let storage = kotoba_storage::MemoryAdapter::new();
//!
//! // Create event stream
//! let event_stream = kotoba_event_stream::EventStream::new(storage);
//!
//! // Execute GQL query
//! let result = kotoba_query_engine::execute_gql("MATCH (n) RETURN n", &event_stream).await;
//! ```
//!
//! ## Crate Organization
//!
//! - **000-core**: Foundation types, error handling, CID
//! - **100-storage**: Storage adapters and persistence
//! - **200-application**: Business logic and domain services
//! - **300-workflow**: Workflow orchestration
//! - **400-language**: Language support (Jsonnet, KotobaScript)
//! - **500-services**: HTTP servers and APIs
//! - **600-deployment**: Deployment and scaling
//! - **900-tools**: Development tools and CLI

pub mod validator;
pub mod runtime;
pub mod dsl;
pub mod ui;
pub mod server;
pub mod wasm_transpiler;
pub mod gql;
pub mod realtime;

// Re-export types from the new crates
pub use kotoba_types::*;
pub use engidb;

/// Result type for EAF-IPG operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for EAF-IPG operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Jsonnet evaluation error: {0}")]
    JsonnetEval(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] engidb::Error),

    #[error("Storage error: {0}")]
    Storage(String),
}
