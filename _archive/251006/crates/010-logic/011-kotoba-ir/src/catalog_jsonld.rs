//! JSON-LD direct manipulation API for Catalog-IR
//!
//! Provides functions to directly manipulate Catalog-IR as JSON-LD Value objects,
//! without requiring Rust struct types.

use serde_json::{json, Value};
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Validate Catalog-IR JSON-LD against SHACL shapes (synchronous wrapper)
#[cfg(feature = "reasoning")]
fn validate_catalog_jsonld_internal(catalog_jsonld: &Value) -> AnyhowResult<()> {
    use crate::shacl::validate_ir_jsonld;
    
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(async {
            let result = validate_ir_jsonld(catalog_jsonld, "CatalogIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Catalog-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    } else {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime for SHACL validation")?;
        rt.block_on(async {
            let result = validate_ir_jsonld(catalog_jsonld, "CatalogIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Catalog-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    }
}

/// Create an empty Catalog-IR as JSON-LD
pub fn create_empty_catalog_jsonld(id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:CatalogIR",
        "kotoba:hasLabels": [],
        "kotoba:hasIndexes": [],
        "kotoba:hasInvariants": [],
    });

    if let Some(catalog_id) = id {
        jsonld["@id"] = json!(catalog_id);
    }

    jsonld
}

/// Get a label definition from catalog JSON-LD
pub fn get_label_def(catalog_jsonld: &Value, label_name: &str) -> Option<Value> {
    catalog_jsonld.get("kotoba:hasLabels")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().find(|label_def| {
                label_def.get("kotoba:labelName")
                    .and_then(|v| v.as_str())
                    .map(|s| s == label_name)
                    .unwrap_or(false)
            })
        })
        .cloned()
}

/// Add a label definition to catalog JSON-LD
pub fn add_label_def(catalog_jsonld: &mut Value, label_def: Value) -> AnyhowResult<()> {
    let labels = catalog_jsonld.get_mut("kotoba:hasLabels")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:hasLabels must be an array")?;

    labels.push(label_def);
    #[cfg(feature = "reasoning")]
    {
        validate_catalog_jsonld_internal(catalog_jsonld)?;
    }
    Ok(())
}

/// Get a property definition from label definition JSON-LD
pub fn get_property_def(label_def_jsonld: &Value, prop_name: &str) -> Option<Value> {
    label_def_jsonld.get("kotoba:hasProperties")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().find(|prop_def| {
                prop_def.get("kotoba:propertyName")
                    .and_then(|v| v.as_str())
                    .map(|s| s == prop_name)
                    .unwrap_or(false)
            })
        })
        .cloned()
}

/// Add a property definition to label definition JSON-LD
pub fn add_property_def(label_def_jsonld: &mut Value, property_def: Value) -> AnyhowResult<()> {
    let properties = label_def_jsonld.get_mut("kotoba:hasProperties")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:hasProperties must be an array")?;

    properties.push(property_def);
    // Note: Property definition changes affect the parent catalog, but we validate
    // at the catalog level when add_label_def is called
    Ok(())
}

/// Get an index definition from catalog JSON-LD
pub fn get_index_def(catalog_jsonld: &Value, index_name: &str) -> Option<Value> {
    catalog_jsonld.get("kotoba:hasIndexes")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().find(|index_def| {
                index_def.get("kotoba:indexName")
                    .and_then(|v| v.as_str())
                    .map(|s| s == index_name)
                    .unwrap_or(false)
            })
        })
        .cloned()
}

/// Add an index definition to catalog JSON-LD
pub fn add_index_def(catalog_jsonld: &mut Value, index_def: Value) -> AnyhowResult<()> {
    let indexes = catalog_jsonld.get_mut("kotoba:hasIndexes")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:hasIndexes must be an array")?;

    indexes.push(index_def);
    #[cfg(feature = "reasoning")]
    {
        validate_catalog_jsonld_internal(catalog_jsonld)?;
    }
    Ok(())
}

/// Get an invariant from catalog JSON-LD
pub fn get_invariant(catalog_jsonld: &Value, invariant_name: &str) -> Option<Value> {
    catalog_jsonld.get("kotoba:hasInvariants")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().find(|invariant| {
                invariant.get("kotoba:invariantName")
                    .and_then(|v| v.as_str())
                    .map(|s| s == invariant_name)
                    .unwrap_or(false)
            })
        })
        .cloned()
}

/// Add an invariant to catalog JSON-LD
pub fn add_invariant(catalog_jsonld: &mut Value, invariant: Value) -> AnyhowResult<()> {
    let invariants = catalog_jsonld.get_mut("kotoba:hasInvariants")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:hasInvariants must be an array")?;

    invariants.push(invariant);
    #[cfg(feature = "reasoning")]
    {
        validate_catalog_jsonld_internal(catalog_jsonld)?;
    }
    Ok(())
}

/// Create a label definition JSON-LD
pub fn create_label_def_jsonld(label_name: &str, id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@type": "kotoba:LabelDef",
        "kotoba:labelName": label_name,
        "kotoba:hasProperties": [],
    });

    if let Some(label_id) = id {
        jsonld["@id"] = json!(label_id);
    }

    jsonld
}

/// Create a property definition JSON-LD
pub fn create_property_def_jsonld(
    property_name: &str,
    property_type: &str,
    nullable: bool,
    default_value: Option<Value>,
    id: Option<&str>,
) -> Value {
    let mut jsonld = json!({
        "@type": "kotoba:PropertyDef",
        "kotoba:propertyName": property_name,
        "kotoba:propertyType": property_type,
        "kotoba:isNullable": nullable,
    });

    if let Some(default) = default_value {
        jsonld["kotoba:defaultValue"] = default;
    }

    if let Some(prop_id) = id {
        jsonld["@id"] = json!(prop_id);
    }

    jsonld
}

/// Create an index definition JSON-LD
pub fn create_index_def_jsonld(
    index_name: &str,
    index_label: &str,
    properties: Vec<String>,
    unique: bool,
    id: Option<&str>,
) -> Value {
    let mut jsonld = json!({
        "@type": "kotoba:IndexDef",
        "kotoba:indexName": index_name,
        "kotoba:indexLabel": index_label,
        "kotoba:indexProperties": properties,
        "kotoba:isUnique": unique,
    });

    if let Some(index_id) = id {
        jsonld["@id"] = json!(index_id);
    }

    jsonld
}

/// Create an invariant JSON-LD
pub fn create_invariant_jsonld(
    invariant_name: &str,
    invariant_expr: &str,
    invariant_message: &str,
    id: Option<&str>,
) -> Value {
    let mut jsonld = json!({
        "@type": "kotoba:Invariant",
        "kotoba:invariantName": invariant_name,
        "kotoba:invariantExpr": invariant_expr,
        "kotoba:invariantMessage": invariant_message,
    });

    if let Some(inv_id) = id {
        jsonld["@id"] = json!(inv_id);
    }

    jsonld
}

/// Validate catalog JSON-LD against SHACL shapes
#[cfg(feature = "reasoning")]
pub async fn validate_catalog_jsonld(catalog_jsonld: &Value) -> AnyhowResult<kotoba_owl_reasoner::shacl::ShaclValidationResult> {
    use kotoba_owl_reasoner::shacl::validate_shacl;
    use std::fs;

    let shape_path = "schemas/catalog-shapes.jsonld";
    let shape_content = fs::read_to_string(shape_path)
        .context(format!("Failed to read SHACL shape file: {}", shape_path))?;
    let shape_jsonld: Value = serde_json::from_str(&shape_content)
        .context("Failed to parse SHACL shape JSON-LD")?;

    validate_shacl(catalog_jsonld, &shape_jsonld).await
        .map_err(|e| anyhow::anyhow!("SHACL validation failed: {}", e))
}

