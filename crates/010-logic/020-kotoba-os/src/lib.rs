//! # Kotoba KotobaOS
//!
//! KotobaOS Kernel + Actor + Mediator pattern implementation for Kotoba.
//! Provides process network execution orchestration using JSON-LD and semantic reasoning.
//!
//! ## Architecture
//!
//! This crate implements the kotobaos pattern:
//! - **Kernel**: Orchestrates process network execution
//! - **Actor**: Performs actions based on capabilities
//! - **Mediator**: Selects appropriate actors using SHACL-based reasoning
//! - **ProcessHandler**: Interprets and orchestrates process networks from stories
//! - **Provenance**: Records execution history in JSON-LD/PROV-O format
//!
//! ## Example
//!
//! ```rust,no_run
//! use kotoba_os::{Kernel, DefaultActor};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a story (JSON-LD format)
//! let story_json = json!({
//!     "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
//!     "@graph": [
//!         {
//!             "@id": "kotoba:process/example",
//!             "@type": "kotoba:Process",
//!             "kotoba:label": "Example Process",
//!             "kotoba:performedBy": "kotoba:performer/actor-1"
//!         }
//!     ]
//! });
//!
//! // Initialize kernel with story
//! let mut kernel = Kernel::new(story_json)?;
//!
//! // Register an actor
//! kernel.register_default_actor(
//!     "kotoba:performer/actor-1",
//!     "kotoba:capability/execution"
//! );
//!
//! // Start orchestration
//! kernel.start().await?;
//! # Ok(())
//! # }
//! ```

pub mod actor;
pub mod kernel;
pub mod mediator;
pub mod process_handler;
pub mod provenance;
pub mod types;

pub use actor::{Actor, ActorTrait, DefaultActor};
pub use kernel::Kernel;
pub use mediator::Mediator;
pub use process_handler::ProcessHandler;
pub use provenance::Provenance;
pub use types::{Process, Resource, Performer, Story, ProvenanceEvent};

#[cfg(feature = "reasoning")]
pub use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};

/// Error types for KotobaOS operations
#[derive(Debug, thiserror::Error)]
pub enum KotobaOsError {
    #[error("JSON-LD parsing error: {0}")]
    JsonLdParse(#[from] serde_json::Error),

    #[error("Story validation error: {0}")]
    StoryValidation(String),

    #[error("Actor selection error: {0}")]
    ActorSelection(String),

    #[error("Process execution error: {0}")]
    ProcessExecution(String),

    #[error("Provenance recording error: {0}")]
    ProvenanceError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, KotobaOsError>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_kernel_creation() {
        let story_json = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@graph": []
        });

        let kernel = Kernel::new(story_json);
        assert!(kernel.is_ok());
    }

    #[tokio::test]
    async fn test_actor_registration() {
        let story_json = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@graph": []
        });

        let mut kernel = Kernel::new(story_json).unwrap();
        kernel.register_default_actor("kotoba:performer/test", "kotoba:capability/test");
        
        assert!(kernel.mediator.has_actor("kotoba:performer/test"));
    }

    #[tokio::test]
    async fn test_process_execution() {
        let story_json = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@graph": [
                {
                    "@id": "kotoba:process/test",
                    "@type": "kotoba:Process",
                    "kotoba:label": "Test Process",
                    "kotoba:performedBy": "kotoba:performer/test-actor"
                }
            ]
        });

        let mut kernel = Kernel::new(story_json).unwrap();
        kernel.register_default_actor("kotoba:performer/test-actor", "kotoba:capability/execution");

        let processes = kernel.story().extract_processes();
        assert_eq!(processes.len(), 1);

        let result = kernel.run_process(&processes[0]).await;
        assert!(result.is_ok());

        // Check provenance was recorded
        let provenance = kernel.provenance_jsonld();
        assert!(provenance.get("@graph").is_some());
    }

    #[tokio::test]
    async fn test_story_orchestration() {
        let story_json = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@graph": [
                {
                    "@id": "kotoba:process/step1",
                    "@type": "kotoba:Process",
                    "kotoba:label": "Step 1",
                    "kotoba:performedBy": "kotoba:performer/actor",
                    "kotoba:next": "kotoba:process/step2"
                },
                {
                    "@id": "kotoba:process/step2",
                    "@type": "kotoba:Process",
                    "kotoba:label": "Step 2",
                    "kotoba:performedBy": "kotoba:performer/actor"
                }
            ]
        });

        let mut kernel = Kernel::new(story_json).unwrap();
        kernel.register_default_actor("kotoba:performer/actor", "kotoba:capability/execution");

        let result = kernel.start().await;
        assert!(result.is_ok());

        // Check provenance has 2 events (one for each process)
        let provenance = kernel.provenance_jsonld();
        if let Some(graph) = provenance.get("@graph").and_then(|g| g.as_array()) {
            assert_eq!(graph.len(), 2);
        }
    }
}
