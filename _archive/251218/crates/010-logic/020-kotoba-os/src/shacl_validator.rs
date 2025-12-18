//! SHACL validation integration for KotobaOS
//!
//! Provides SHACL-based validation for Process, Resource, and Performer types.

use crate::types::{Process, Resource, Performer};
use crate::{KotobaOsError, Result};
use serde_json::Value;
use tracing::{info, warn};

/// SHACL validator for KotobaOS types
pub struct ShaclValidator {
    /// Enable strict validation (fail on validation errors)
    strict: bool,
}

impl ShaclValidator {
    /// Create a new SHACL validator
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a validator with strict mode
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Validate a Process instance
    #[cfg(feature = "reasoning")]
    pub async fn validate_process(&self, process: &Process) -> Result<()> {
        use kotoba_owl_reasoner::{validate_process_shape, default_process_shape};

        let process_jsonld = serde_json::to_value(process)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize process: {}", e)))?;

        let shape_jsonld = default_process_shape();

        let validation_result = validate_process_shape(&process_jsonld, &shape_jsonld).await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("SHACL validation error: {}", e)))?;

        if !validation_result.valid {
            let error_msg = format!("SHACL validation failed for process {}: {:?}",
                                    process.id, validation_result.errors);
            
            if self.strict {
                return Err(KotobaOsError::ProcessExecution(error_msg));
            } else {
                warn!("[ShaclValidator] {}", error_msg);
            }
        } else {
            info!("[ShaclValidator] Process {} passed SHACL validation", process.id);
        }

        Ok(())
    }

    /// Validate a Resource instance
    #[cfg(feature = "reasoning")]
    pub async fn validate_resource(&self, resource: &Resource) -> Result<()> {
        use kotoba_owl_reasoner::{validate_resource_shape, default_resource_shape};

        let resource_jsonld = serde_json::to_value(resource)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize resource: {}", e)))?;

        let shape_jsonld = default_resource_shape();

        let validation_result = validate_resource_shape(&resource_jsonld, &shape_jsonld).await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("SHACL validation error: {}", e)))?;

        if !validation_result.valid {
            let error_msg = format!("SHACL validation failed for resource {}: {:?}",
                                    resource.id, validation_result.errors);
            
            if self.strict {
                return Err(KotobaOsError::ProcessExecution(error_msg));
            } else {
                warn!("[ShaclValidator] {}", error_msg);
            }
        } else {
            info!("[ShaclValidator] Resource {} passed SHACL validation", resource.id);
        }

        Ok(())
    }

    /// Validate a Performer instance
    #[cfg(feature = "reasoning")]
    pub async fn validate_performer(&self, performer: &Performer) -> Result<()> {
        use kotoba_owl_reasoner::{validate_performer_shape, default_performer_shape};

        let performer_jsonld = serde_json::to_value(performer)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize performer: {}", e)))?;

        let shape_jsonld = default_performer_shape();

        let validation_result = validate_performer_shape(&performer_jsonld, &shape_jsonld).await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("SHACL validation error: {}", e)))?;

        if !validation_result.valid {
            let error_msg = format!("SHACL validation failed for performer {}: {:?}",
                                    performer.id, validation_result.errors);
            
            if self.strict {
                return Err(KotobaOsError::ProcessExecution(error_msg));
            } else {
                warn!("[ShaclValidator] {}", error_msg);
            }
        } else {
            info!("[ShaclValidator] Performer {} passed SHACL validation", performer.id);
        }

        Ok(())
    }

    /// Validate without feature flag (no-op)
    #[cfg(not(feature = "reasoning"))]
    pub async fn validate_process(&self, _process: &Process) -> Result<()> {
        Ok(())
    }

    #[cfg(not(feature = "reasoning"))]
    pub async fn validate_resource(&self, _resource: &Resource) -> Result<()> {
        Ok(())
    }

    #[cfg(not(feature = "reasoning"))]
    pub async fn validate_performer(&self, _performer: &Performer) -> Result<()> {
        Ok(())
    }
}

impl Default for ShaclValidator {
    fn default() -> Self {
        Self::new()
    }
}

