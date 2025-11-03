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
}

impl Actor {
    /// Create a new actor with an ID and capability
    pub fn new(id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capability: capability.into(),
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
}

