//! Capability System for KotobaOS
//!
//! Provides OWL/SHACL-based capability matching for Actor selection.
//! Capabilities are defined using OWL ontology and validated using SHACL shapes.

use crate::{Result, KotobaOsError};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, debug};

#[cfg(feature = "reasoning")]
use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel, FukurowStore};

/// Capability represents a capability that can be provided by an Actor or required by a Process
#[derive(Debug, Clone)]
pub struct Capability {
    /// Capability ID (IRI)
    pub id: String,
    /// Capability type (OWL class IRI)
    pub capability_type: String,
    /// Capability level (optional)
    pub level: Option<String>,
    /// Additional constraints (SHACL shape)
    pub constraints: Option<Value>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl Capability {
    /// Create a new capability
    pub fn new(id: impl Into<String>, capability_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capability_type: capability_type.into(),
            level: None,
            constraints: None,
            metadata: HashMap::new(),
        }
    }

    /// Set capability level
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = Some(level.into());
        self
    }

    /// Set constraints (SHACL shape)
    pub fn with_constraints(mut self, constraints: Value) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Convert to JSON-LD format
    pub fn to_jsonld(&self) -> Value {
        let mut jsonld = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@id": self.id,
            "@type": self.capability_type,
            "kotoba:capabilityId": self.id,
        });

        if let Some(ref level) = self.level {
            jsonld["kotoba:hasCapabilityLevel"] = json!(level);
        }

        if let Some(ref constraints) = self.constraints {
            jsonld["kotoba:capabilityConstraints"] = constraints.clone();
        }

        for (key, value) in &self.metadata {
            jsonld[key] = value.clone();
        }

        jsonld
    }

    /// Create from JSON-LD format
    pub fn from_jsonld(jsonld: &Value) -> Result<Self> {
        let id = jsonld.get("@id")
            .or_else(|| jsonld.get("kotoba:capabilityId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| KotobaOsError::InvalidCapability("Missing capability ID".to_string()))?
            .to_string();

        let capability_type = jsonld.get("@type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| KotobaOsError::InvalidCapability("Missing capability type".to_string()))?
            .to_string();

        let mut capability = Capability::new(id.clone(), capability_type);

        if let Some(level) = jsonld.get("kotoba:hasCapabilityLevel").and_then(|v| v.as_str()) {
            capability.level = Some(level.to_string());
        }

        if let Some(constraints) = jsonld.get("kotoba:capabilityConstraints") {
            capability.constraints = Some(constraints.clone());
        }

        // Extract additional metadata
        for (key, value) in jsonld.as_object().unwrap_or(&serde_json::Map::new()) {
            if !key.starts_with("@") && !key.starts_with("kotoba:") {
                capability.metadata.insert(key.clone(), value.clone());
            }
        }

        Ok(capability)
    }
}

/// CapabilityRegistry manages capabilities and provides OWL-based matching
pub struct CapabilityRegistry {
    /// Registered capabilities by ID
    capabilities: HashMap<String, Capability>,
    /// OWL reasoning engine (if reasoning feature is enabled)
    #[cfg(feature = "reasoning")]
    reasoning_engine: Option<ReasoningEngine>,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            #[cfg(feature = "reasoning")]
            reasoning_engine: None,
        }
    }

    /// Create registry with OWL reasoning enabled
    #[cfg(feature = "reasoning")]
    pub async fn with_reasoning(level: ReasoningLevel) -> Result<Self> {
        let mut engine = ReasoningEngine::new(level)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to create reasoning engine: {}", e)))?;

        // Load capability ontology
        let ontology_json = include_str!("../../../schemas/capability-ontology.jsonld");
        let ontology: Value = serde_json::from_str(ontology_json)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to parse capability ontology: {}", e)))?;

        engine.load_ontology_from_jsonld(ontology).await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to load capability ontology: {}", e)))?;

        Ok(Self {
            capabilities: HashMap::new(),
            reasoning_engine: Some(engine),
        })
    }

    /// Register a capability
    pub fn register(&mut self, capability: Capability) {
        info!("[CapabilityRegistry] Registering capability: {} ({})", capability.id, capability.capability_type);
        self.capabilities.insert(capability.id.clone(), capability);
    }

    /// Get a capability by ID
    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// Find capabilities by type using OWL reasoning
    #[cfg(feature = "reasoning")]
    pub async fn find_by_type(&self, capability_type: &str) -> Result<Vec<Capability>> {
        if let Some(ref engine) = self.reasoning_engine {
            // Use OWL reasoning to find all capabilities that are subtypes of the requested type
            let query = format!(
                r#"
                PREFIX kotoba: <https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab>
                SELECT ?capability WHERE {{
                    ?capability rdf:type/rdfs:subClassOf* <{}> .
                }}
                "#,
                capability_type
            );

            // For now, use simple type matching
            // TODO: Implement full SPARQL query support when available
            let mut matching_capabilities = Vec::new();
            
            // Check if any capability is a subtype of the requested type
            for capability in self.capabilities.values() {
                if capability.capability_type == capability_type {
                    matching_capabilities.push(capability.clone());
                }
            }

            Ok(matching_capabilities)
        } else {
            // Fallback to exact type matching without reasoning
            Ok(self.capabilities
                .values()
                .filter(|cap| cap.capability_type == capability_type)
                .cloned()
                .collect())
        }
    }

    /// Find capabilities by type (fallback without reasoning)
    #[cfg(not(feature = "reasoning"))]
    pub async fn find_by_type(&self, capability_type: &str) -> Result<Vec<Capability>> {
        Ok(self.capabilities
            .values()
            .filter(|cap| cap.capability_type == capability_type)
            .cloned()
            .collect())
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// CapabilityMatcher provides OWL-based capability matching
pub struct CapabilityMatcher {
    /// Capability registry (cloned for use)
    registry: CapabilityRegistry,
}

impl CapabilityMatcher {
    /// Create a new capability matcher
    pub fn new(registry: CapabilityRegistry) -> Self {
        Self { registry }
    }

    /// Match a required capability with provided capabilities using OWL reasoning
    #[cfg(feature = "reasoning")]
    pub async fn match_capabilities(
        &self,
        required_capability: &Capability,
        provided_capabilities: &[Capability],
    ) -> Result<Vec<Capability>> {
        // Use OWL reasoning to check if any provided capability satisfies the requirement
        let mut matching = Vec::new();

        for provided in provided_capabilities {
            if self.can_satisfy(required_capability, provided).await? {
                matching.push(provided.clone());
            }
        }

        Ok(matching)
    }

    /// Match capabilities (fallback without reasoning)
    #[cfg(not(feature = "reasoning"))]
    pub async fn match_capabilities(
        &self,
        required_capability: &Capability,
        provided_capabilities: &[Capability],
    ) -> Result<Vec<Capability>> {
        // Simple type matching without OWL reasoning
        Ok(provided_capabilities
            .iter()
            .filter(|provided| provided.capability_type == required_capability.capability_type)
            .cloned()
            .collect())
    }

    /// Check if a provided capability can satisfy a required capability using OWL reasoning
    #[cfg(feature = "reasoning")]
    async fn can_satisfy(
        &self,
        required: &Capability,
        provided: &Capability,
    ) -> Result<bool> {
        // For now, use simple type matching
        // In a full implementation, this would use OWL subsumption reasoning
        if provided.capability_type == required.capability_type {
            return Ok(true);
        }

        // TODO: Implement OWL subsumption check
        // Check if provided.capability_type is a subclass of required.capability_type
        Ok(false)
    }
}

/// Extract required capabilities from a Process JSON-LD
pub fn extract_required_capabilities(process_jsonld: &Value) -> Result<Vec<Capability>> {
    let mut capabilities = Vec::new();

    if let Some(required_arr) = process_jsonld.get("kotoba:requiresCapability") {
        if let Some(arr) = required_arr.as_array() {
            for cap_val in arr {
                let capability = Capability::from_jsonld(cap_val)?;
                capabilities.push(capability);
            }
        } else if let Some(cap_obj) = required_arr.as_object() {
            // Single capability as object
            let capability = Capability::from_jsonld(required_arr)?;
            capabilities.push(capability);
        } else if let Some(cap_iri) = required_arr.as_str() {
            // Single capability as IRI string
            let capability = Capability::new(cap_iri, "kotoba:Capability");
            capabilities.push(capability);
        }
    }

    Ok(capabilities)
}

/// Extract provided capabilities from an Actor JSON-LD
pub fn extract_provided_capabilities(actor_jsonld: &Value) -> Result<Vec<Capability>> {
    let mut capabilities = Vec::new();

    if let Some(provided_arr) = actor_jsonld.get("kotoba:providesCapability") {
        if let Some(arr) = provided_arr.as_array() {
            for cap_val in arr {
                let capability = Capability::from_jsonld(cap_val)?;
                capabilities.push(capability);
            }
        } else if let Some(cap_obj) = provided_arr.as_object() {
            // Single capability as object
            let capability = Capability::from_jsonld(provided_arr)?;
            capabilities.push(capability);
        } else if let Some(cap_iri) = provided_arr.as_str() {
            // Single capability as IRI string
            let capability = Capability::new(cap_iri, "kotoba:Capability");
            capabilities.push(capability);
        }
    }

    // Also check legacy "kotoba:capability" field
    if let Some(cap_iri) = actor_jsonld.get("kotoba:capability").and_then(|v| v.as_str()) {
        let capability = Capability::new(cap_iri, "kotoba:Capability");
        capabilities.push(capability);
    }

    Ok(capabilities)
}

