//! Actor implementation for KotobaOS
//!
//! Actors are components that perform actions based on their capabilities.
//! They resolve I/O from SHACL shapes and execute processes.

use crate::types::{Process, Resource};
use crate::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

/// Actor represents a component that can perform actions
#[derive(Debug, Clone)]
pub struct Actor {
    /// Unique identifier for the actor
    pub id: String,

    /// Capability IRI describing what the actor can do
    pub capability: String,

    /// Optional SHACL shape describing the actor's capability constraints
    #[cfg(feature = "reasoning")]
    pub shacl_shape: Option<Value>,
}

impl Actor {
    /// Create a new actor with an ID and capability
    pub fn new(id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capability: capability.into(),
            #[cfg(feature = "reasoning")]
            shacl_shape: None,
        }
    }

    /// Create an actor with SHACL shape
    #[cfg(feature = "reasoning")]
    pub fn with_shape(
        id: impl Into<String>,
        capability: impl Into<String>,
        shape: Value,
    ) -> Self {
        Self {
            id: id.into(),
            capability: capability.into(),
            shacl_shape: Some(shape),
        }
    }

    /// Resolve I/O from a process JSON-LD structure
    /// This extracts input/output from SHACL shape properties
    pub fn resolve_io(&self, process: &Process) -> HashMap<String, Value> {
        let mut io = HashMap::new();

        // Extract used resources as input
        if let Some(used) = &process.used {
            io.insert("used".to_string(), Value::Array(
                used.iter().map(|iri| Value::String(iri.clone())).collect()
            ));
        }

        // Extract generated resources as output specification
        if let Some(generated) = &process.generated {
            io.insert("generated".to_string(), Value::Array(
                generated.iter().map(|iri| Value::String(iri.clone())).collect()
            ));
        }

        // Extract additional properties as input
        for (key, value) in &process.additional {
            if !key.starts_with("@") && !key.starts_with("kotoba:") {
                io.insert(key.clone(), value.clone());
            }
        }

        io
    }

    /// Wrap output resource with provenance metadata
    pub fn wrap_output(&self, output: Resource) -> Resource {
        let mut wrapped = output;
        
        // Add provenance metadata if not already present
        if !wrapped.additional.contains_key("prov:wasGeneratedBy") {
            wrapped.additional.insert(
                "prov:wasGeneratedBy".to_string(),
                Value::String(self.id.clone()),
            );
        }

        wrapped
    }
}

/// Trait for actors that can perform processes
#[async_trait]
pub trait ActorTrait: Send + Sync {
    /// Perform a process and return a result resource
    async fn perform(&self, process: &Process) -> Result<Resource>;

    /// Get the actor's ID
    fn id(&self) -> &str;

    /// Get the actor's capability
    fn capability(&self) -> &str;

    /// Get the actor's SHACL shape (if available)
    #[cfg(feature = "reasoning")]
    fn shacl_shape(&self) -> Option<&Value> {
        None
    }

    /// Calculate compatibility score with a process (0.0 to 1.0)
    /// Higher score means better match
    #[cfg(feature = "reasoning")]
    async fn compatibility_score(&self, process: &Process) -> f64 {
        // Default implementation: simple capability matching
        // Future: SHACL-based semantic matching
        if self.capability() == process.performed_by {
            1.0
        } else if process.performed_by.contains(self.capability()) {
            0.8
        } else {
            0.0
        }
    }
}

/// Default actor implementation
pub struct DefaultActor {
    actor: Actor,
}

impl DefaultActor {
    pub fn new(id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            actor: Actor::new(id, capability),
        }
    }

    /// Create with SHACL shape
    #[cfg(feature = "reasoning")]
    pub fn with_shape(
        id: impl Into<String>,
        capability: impl Into<String>,
        shape: Value,
    ) -> Self {
        Self {
            actor: Actor::with_shape(id, capability, shape),
        }
    }
}

#[async_trait]
impl ActorTrait for DefaultActor {
    async fn perform(&self, process: &Process) -> Result<Resource> {
        info!("[Actor: {}] performing process: {}", self.actor.id, 
              process.label.as_deref().unwrap_or(&process.id));

        // Resolve I/O
        let io = self.actor.resolve_io(process);

        // Create a simple result resource
        let result = Resource {
            id: format!("kotoba:resource/result-{}", uuid::Uuid::new_v4()),
            type_: "kotoba:Resource".to_string(),
            context: process.context.clone(),
            label: Some(format!("Result of {}", 
                process.label.as_deref().unwrap_or(&process.id))),
            additional: io,
        };

        Ok(self.actor.wrap_output(result))
    }

    fn id(&self) -> &str {
        &self.actor.id
    }

    fn capability(&self) -> &str {
        &self.actor.capability
    }

    #[cfg(feature = "reasoning")]
    fn shacl_shape(&self) -> Option<&Value> {
        self.actor.shacl_shape.as_ref()
    }

    #[cfg(feature = "reasoning")]
    async fn compatibility_score(&self, process: &Process) -> f64 {
        // Check if SHACL shape is available
        if let Some(shape) = &self.actor.shacl_shape {
            // TODO: Implement SHACL-based semantic matching
            // For now, use simple capability matching
            return self.actor.capability_score(process);
        }

        // Fallback to simple capability matching
        self.actor.capability_score(process)
    }
}

impl Actor {
    /// Calculate capability score (simple matching)
    fn capability_score(&self, process: &Process) -> f64 {
        if self.capability == process.performed_by {
            1.0
        } else if process.performed_by.contains(&self.capability) {
            0.8
        } else if self.capability.contains(&process.performed_by) {
            0.6
        } else {
            0.0
        }
    }
}
