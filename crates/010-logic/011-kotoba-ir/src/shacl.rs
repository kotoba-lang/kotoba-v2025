//! SHACL validation for JSON-LD IRs

use serde_json::Value;
use anyhow::{Context, Result as AnyhowResult};

/// Validate IR JSON-LD against SHACL shapes
#[cfg(feature = "reasoning")]
pub async fn validate_ir_jsonld(ir_jsonld: &Value, ir_type: &str) -> AnyhowResult<kotoba_owl_reasoner::shacl::ShaclValidationResult> {
    use kotoba_owl_reasoner::shacl::validate_shacl;
    use std::fs;

    // Load appropriate SHACL shape based on IR type
    let shape_path = match ir_type {
        "RuleIR" => "schemas/ir-shapes.jsonld",
        "QueryIR" => "schemas/ir-shapes.jsonld",
        "PatchIR" => "schemas/ir-shapes.jsonld",
        "StrategyIR" => "schemas/ir-shapes.jsonld",
        "CatalogIR" => "schemas/catalog-shapes.jsonld",
        _ => return Err(anyhow::anyhow!("Unknown IR type: {}", ir_type)),
    };

    let shape_content = fs::read_to_string(shape_path)
        .context(format!("Failed to read SHACL shape file: {}", shape_path))?;
    let shape_jsonld: Value = serde_json::from_str(&shape_content)
        .context("Failed to parse SHACL shape JSON-LD")?;

    validate_shacl(ir_jsonld, &shape_jsonld).await
        .map_err(|e| anyhow::anyhow!("SHACL validation failed: {}", e))
}

