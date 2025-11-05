//! JSON-LD direct manipulation API for Patch-IR
//!
//! Provides functions to directly manipulate Patch-IR as JSON-LD Value objects,
//! without requiring Rust struct types.

use serde_json::{json, Value};
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Validate Patch-IR JSON-LD against SHACL shapes (synchronous wrapper)
#[cfg(feature = "reasoning")]
fn validate_patch_jsonld(patch_jsonld: &Value) -> AnyhowResult<()> {
    use crate::shacl::validate_ir_jsonld;
    
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(async {
            let result = validate_ir_jsonld(patch_jsonld, "PatchIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Patch-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    } else {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime for SHACL validation")?;
        rt.block_on(async {
            let result = validate_ir_jsonld(patch_jsonld, "PatchIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Patch-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    }
}

/// Create an empty Patch-IR as JSON-LD
pub fn create_empty_patch_jsonld(id: Option<&str>) -> Value {
    let mut patch = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:PatchIR",
        "kotoba:adds": {
            "kotoba:vertices": [],
            "kotoba:edges": [],
        },
        "kotoba:dels": {
            "kotoba:vertices": [],
            "kotoba:edges": [],
        },
        "kotoba:updates": {
            "kotoba:props": [],
            "kotoba:relinks": [],
        },
    });

    if let Some(patch_id) = id {
        patch["@id"] = json!(patch_id);
    }

    patch
}

/// Add a vertex to Patch-IR JSON-LD
pub fn add_vertex(patch_jsonld: &mut Value, vertex_id: &str, labels: Vec<&str>, props: Option<Value>) -> AnyhowResult<()> {
    let vertices = patch_jsonld
        .get_mut("kotoba:adds")
        .and_then(|v| v.get_mut("kotoba:vertices"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:adds.kotoba:vertices must be an array")?;

    let mut vertex = json!({
        "kotoba:id": vertex_id,
        "kotoba:labels": labels,
    });

    if let Some(p) = props {
        vertex["kotoba:props"] = p;
    }

    vertices.push(vertex);
    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Add an edge to Patch-IR JSON-LD
pub fn add_edge(patch_jsonld: &mut Value, edge_id: &str, src: &str, dst: &str, label: &str, props: Option<Value>) -> AnyhowResult<()> {
    let edges = patch_jsonld
        .get_mut("kotoba:adds")
        .and_then(|v| v.get_mut("kotoba:edges"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:adds.kotoba:edges must be an array")?;

    let mut edge = json!({
        "kotoba:id": edge_id,
        "kotoba:src": src,
        "kotoba:dst": dst,
        "kotoba:label": label,
    });

    if let Some(p) = props {
        edge["kotoba:props"] = p;
    }

    edges.push(edge);
    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Delete a vertex from Patch-IR JSON-LD
pub fn delete_vertex(patch_jsonld: &mut Value, vertex_id: &str) -> AnyhowResult<()> {
    let vertices = patch_jsonld
        .get_mut("kotoba:dels")
        .and_then(|v| v.get_mut("kotoba:vertices"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:dels.kotoba:vertices must be an array")?;

    vertices.push(json!(vertex_id));
    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Delete an edge from Patch-IR JSON-LD
pub fn delete_edge(patch_jsonld: &mut Value, edge_id: &str) -> AnyhowResult<()> {
    let edges = patch_jsonld
        .get_mut("kotoba:dels")
        .and_then(|v| v.get_mut("kotoba:edges"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:dels.kotoba:edges must be an array")?;

    edges.push(json!(edge_id));
    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Update a property in Patch-IR JSON-LD
pub fn update_property(patch_jsonld: &mut Value, element_id: &str, key: &str, value: Value) -> AnyhowResult<()> {
    let props = patch_jsonld
        .get_mut("kotoba:updates")
        .and_then(|v| v.get_mut("kotoba:props"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:updates.kotoba:props must be an array")?;

    props.push(json!({
        "kotoba:id": element_id,
        "kotoba:key": key,
        "kotoba:value": value,
    }));

    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Relink an edge in Patch-IR JSON-LD
pub fn relink_edge(patch_jsonld: &mut Value, edge_id: &str, new_src: Option<&str>, new_dst: Option<&str>) -> AnyhowResult<()> {
    let relinks = patch_jsonld
        .get_mut("kotoba:updates")
        .and_then(|v| v.get_mut("kotoba:relinks"))
        .and_then(|v| v.as_array_mut())
        .context("kotoba:updates.kotoba:relinks must be an array")?;

    let mut relink = json!({
        "kotoba:edgeId": edge_id,
    });

    if let Some(src) = new_src {
        relink["kotoba:newSrc"] = json!(src);
    }

    if let Some(dst) = new_dst {
        relink["kotoba:newDst"] = json!(dst);
    }

    relinks.push(relink);
    #[cfg(feature = "reasoning")]
    {
        validate_patch_jsonld(patch_jsonld)?;
    }
    Ok(())
}

/// Get adds from Patch-IR JSON-LD
pub fn get_adds(patch_jsonld: &Value) -> Option<Value> {
    patch_jsonld.get("kotoba:adds").cloned()
}

/// Get dels from Patch-IR JSON-LD
pub fn get_dels(patch_jsonld: &Value) -> Option<Value> {
    patch_jsonld.get("kotoba:dels").cloned()
}

/// Get updates from Patch-IR JSON-LD
pub fn get_updates(patch_jsonld: &Value) -> Option<Value> {
    patch_jsonld.get("kotoba:updates").cloned()
}

/// Check if patch is empty
pub fn is_empty(patch_jsonld: &Value) -> bool {
    let adds = patch_jsonld.get("kotoba:adds")
        .and_then(|v| v.get("kotoba:vertices"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    let edges_empty = patch_jsonld.get("kotoba:adds")
        .and_then(|v| v.get("kotoba:edges"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    let dels_empty = patch_jsonld.get("kotoba:dels")
        .and_then(|v| v.get("kotoba:vertices"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    let edge_dels_empty = patch_jsonld.get("kotoba:dels")
        .and_then(|v| v.get("kotoba:edges"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    let props_empty = patch_jsonld.get("kotoba:updates")
        .and_then(|v| v.get("kotoba:props"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    let relinks_empty = patch_jsonld.get("kotoba:updates")
        .and_then(|v| v.get("kotoba:relinks"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    adds && edges_empty && dels_empty && edge_dels_empty && props_empty && relinks_empty
}

