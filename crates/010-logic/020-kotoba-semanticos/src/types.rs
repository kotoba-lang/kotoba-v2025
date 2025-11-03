//! Type definitions for SemanticOS components
//!
//! Defines Process, Resource, Performer, Story, and ProvenanceEvent types
//! based on semanticos ontology in JSON-LD format.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Process represents a unit of execution in the process network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    /// IRI identifier (@id)
    #[serde(rename = "@id")]
    pub id: String,

    /// Type (@type)
    #[serde(rename = "@type")]
    #[serde(default = "default_process_type")]
    pub type_: String,

    /// Context (@context) - optional
    #[serde(rename = "@context")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Label (kotoba:label or rdfs:label)
    #[serde(rename = "kotoba:label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Performer IRI (kotoba:performedBy)
    #[serde(rename = "kotoba:performedBy")]
    pub performed_by: String,

    /// Used resources (kotoba:used)
    #[serde(rename = "kotoba:used")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<Vec<String>>,

    /// Generated resources (kotoba:generated)
    #[serde(rename = "kotoba:generated")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<Vec<String>>,

    /// Next process IRI (kotoba:next)
    #[serde(rename = "kotoba:next")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

fn default_process_type() -> String {
    "kotoba:Process".to_string()
}

/// Resource represents any entity in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// IRI identifier (@id)
    #[serde(rename = "@id")]
    pub id: String,

    /// Type (@type)
    #[serde(rename = "@type")]
    #[serde(default = "default_resource_type")]
    pub type_: String,

    /// Context (@context) - optional
    #[serde(rename = "@context")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Label (kotoba:label or rdfs:label)
    #[serde(rename = "kotoba:label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

fn default_resource_type() -> String {
    "kotoba:Resource".to_string()
}

/// Performer represents an entity that can perform processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performer {
    /// IRI identifier (@id)
    #[serde(rename = "@id")]
    pub id: String,

    /// Type (@type)
    #[serde(rename = "@type")]
    #[serde(default = "default_performer_type")]
    pub type_: String,

    /// Context (@context) - optional
    #[serde(rename = "@context")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Label (kotoba:label or rdfs:label)
    #[serde(rename = "kotoba:label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

fn default_performer_type() -> String {
    "kotoba:Performer".to_string()
}

/// Story represents a process network graph in JSON-LD format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    /// Context (@context)
    #[serde(rename = "@context")]
    pub context: Value,

    /// Graph (@graph) containing processes, resources, and performers
    #[serde(rename = "@graph")]
    pub graph: Vec<Value>,
}

impl Story {
    /// Parse Story from JSON-LD Value
    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Extract processes from the graph
    pub fn extract_processes(&self) -> Vec<Process> {
        self.graph
            .iter()
            .filter_map(|item| {
                if let Some(type_) = item.get("@type").and_then(|v| v.as_str()) {
                    if type_ == "kotoba:Process" || type_.ends_with("Process") {
                        serde_json::from_value(item.clone()).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract resources from the graph
    pub fn extract_resources(&self) -> Vec<Resource> {
        self.graph
            .iter()
            .filter_map(|item| {
                if let Some(type_) = item.get("@type").and_then(|v| v.as_str()) {
                    if type_ == "kotoba:Resource" || type_.ends_with("Resource") {
                        serde_json::from_value(item.clone()).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract performers from the graph
    pub fn extract_performers(&self) -> Vec<Performer> {
        self.graph
            .iter()
            .filter_map(|item| {
                if let Some(type_) = item.get("@type").and_then(|v| v.as_str()) {
                    if type_ == "kotoba:Performer" || type_.ends_with("Performer") {
                        serde_json::from_value(item.clone()).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

/// ProvenanceEvent represents an execution history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEvent {
    /// IRI identifier (@id)
    #[serde(rename = "@id")]
    pub id: String,

    /// Type (@type)
    #[serde(rename = "@type")]
    #[serde(default = "default_provenance_type")]
    pub type_: String,

    /// Context (@context) - optional
    #[serde(rename = "@context")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Process that generated this event (prov:wasGeneratedBy)
    #[serde(rename = "prov:wasGeneratedBy")]
    pub was_generated_by: String,

    /// Actor associated with this event (prov:wasAssociatedWith)
    #[serde(rename = "prov:wasAssociatedWith")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was_associated_with: Option<String>,

    /// Used resources (prov:used)
    #[serde(rename = "prov:used")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<Vec<String>>,

    /// Generated resources (prov:generated)
    #[serde(rename = "prov:generated")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<Vec<String>>,

    /// End time (prov:endedAtTime)
    #[serde(rename = "prov:endedAtTime")]
    pub ended_at_time: String,

    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

fn default_provenance_type() -> String {
    "kotoba:ProvenanceEvent".to_string()
}

