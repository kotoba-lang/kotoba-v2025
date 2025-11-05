//! JSON-LD direct manipulation API for Query-IR
//!
//! Provides functions to directly manipulate Query-IR as JSON-LD Value objects,
//! without requiring Rust struct types.

use serde_json::{json, Value};
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Validate Query-IR JSON-LD against SHACL shapes (synchronous wrapper)
#[cfg(feature = "reasoning")]
fn validate_query_jsonld(query_jsonld: &Value) -> AnyhowResult<()> {
    use crate::shacl::validate_ir_jsonld;
    
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(async {
            let result = validate_ir_jsonld(query_jsonld, "QueryIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Query-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    } else {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime for SHACL validation")?;
        rt.block_on(async {
            let result = validate_ir_jsonld(query_jsonld, "QueryIR").await?;
            if !result.valid {
                return Err(anyhow::anyhow!(
                    "SHACL validation failed for Query-IR: {:?}",
                    result.errors
                ));
            }
            Ok(())
        })
    }
}

/// Create an empty Query-IR as JSON-LD
pub fn create_empty_query_jsonld(id: Option<&str>) -> Value {
    let mut query = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:QueryIR",
        "kotoba:plan": null,
    });

    if let Some(query_id) = id {
        query["@id"] = json!(query_id);
    }

    query
}

/// Get plan from Query-IR JSON-LD
pub fn get_plan(query_jsonld: &Value) -> Option<Value> {
    query_jsonld.get("kotoba:plan").cloned()
}

/// Set plan in Query-IR JSON-LD
pub fn set_plan(query_jsonld: &mut Value, plan: Value) -> AnyhowResult<()> {
    query_jsonld["kotoba:plan"] = plan;
    #[cfg(feature = "reasoning")]
    {
        validate_query_jsonld(query_jsonld)?;
    }
    Ok(())
}

/// Create a NodeScan logical operator JSON-LD
pub fn create_node_scan(label: &str, as_: &str, props: Option<Value>) -> Value {
    let mut op = json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "NodeScan",
        "kotoba:label": label,
        "kotoba:as": as_,
    });

    if let Some(p) = props {
        op["kotoba:props"] = p;
    }

    op
}

/// Create a Filter logical operator JSON-LD
pub fn create_filter(pred: Value, input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Filter",
        "kotoba:pred": pred,
        "kotoba:input": input,
    })
}

/// Create an Expand logical operator JSON-LD
pub fn create_expand(edge: Value, to_as: &str, from: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Expand",
        "kotoba:edge": edge,
        "kotoba:toAs": to_as,
        "kotoba:from": from,
    })
}

/// Create a Join logical operator JSON-LD
pub fn create_join(left: Value, right: Value, on: Vec<&str>) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Join",
        "kotoba:left": left,
        "kotoba:right": right,
        "kotoba:on": on,
    })
}

/// Create a Project logical operator JSON-LD
pub fn create_project(cols: Vec<&str>, input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Project",
        "kotoba:cols": cols,
        "kotoba:input": input,
    })
}

/// Create a Group logical operator JSON-LD
pub fn create_group(keys: Vec<&str>, aggregations: Value, input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Group",
        "kotoba:keys": keys,
        "kotoba:aggregations": aggregations,
        "kotoba:input": input,
    })
}

/// Create a Sort logical operator JSON-LD
pub fn create_sort(keys: Value, input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Sort",
        "kotoba:keys": keys,
        "kotoba:input": input,
    })
}

/// Create a Limit logical operator JSON-LD
pub fn create_limit(count: usize, input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Limit",
        "kotoba:count": count,
        "kotoba:input": input,
    })
}

/// Create a Distinct logical operator JSON-LD
pub fn create_distinct(input: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "Distinct",
        "kotoba:input": input,
    })
}

/// Create an IndexScan logical operator JSON-LD
pub fn create_index_scan(label: &str, as_: &str, index: &str, value: Value) -> Value {
    json!({
        "@type": "kotoba:LogicalOp",
        "kotoba:op": "IndexScan",
        "kotoba:label": label,
        "kotoba:as": as_,
        "kotoba:index": index,
        "kotoba:value": value,
    })
}

/// Get operator type from logical operator JSON-LD
pub fn get_operator_type(op_jsonld: &Value) -> Option<String> {
    op_jsonld.get("kotoba:op")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

