//! JSON-LD direct manipulation API for Rule-IR
//!
//! Provides functions to directly manipulate Rule-IR as JSON-LD Value objects,
//! without requiring Rust struct types.

use serde_json::{json, Value};
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Create an empty Rule-IR as JSON-LD
pub fn create_empty_rule_jsonld(id: Option<&str>, name: &str) -> Value {
    let mut rule = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:RuleIR",
        "kotoba:name": name,
        "kotoba:lhs": {
            "kotoba:nodes": [],
            "kotoba:edges": [],
        },
        "kotoba:context": {
            "kotoba:nodes": [],
            "kotoba:edges": [],
        },
        "kotoba:rhs": {
            "kotoba:nodes": [],
            "kotoba:edges": [],
        },
        "kotoba:types": {},
        "kotoba:nacs": [],
        "kotoba:guards": [],
    });

    if let Some(rule_id) = id {
        rule["@id"] = json!(rule_id);
    }

    rule
}

/// Get rule name from Rule-IR JSON-LD
pub fn get_rule_name(rule_jsonld: &Value) -> Option<String> {
    rule_jsonld.get("kotoba:name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Set rule name in Rule-IR JSON-LD
pub fn set_rule_name(rule_jsonld: &mut Value, name: &str) -> AnyhowResult<()> {
    rule_jsonld["kotoba:name"] = json!(name);
    Ok(())
}

/// Get LHS graph pattern from Rule-IR JSON-LD
pub fn get_lhs(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:lhs").cloned()
}

/// Set LHS graph pattern in Rule-IR JSON-LD
pub fn set_lhs(rule_jsonld: &mut Value, lhs: Value) -> AnyhowResult<()> {
    rule_jsonld["kotoba:lhs"] = lhs;
    Ok(())
}

/// Get RHS graph pattern from Rule-IR JSON-LD
pub fn get_rhs(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:rhs").cloned()
}

/// Set RHS graph pattern in Rule-IR JSON-LD
pub fn set_rhs(rule_jsonld: &mut Value, rhs: Value) -> AnyhowResult<()> {
    rule_jsonld["kotoba:rhs"] = rhs;
    Ok(())
}

/// Get context graph pattern from Rule-IR JSON-LD
pub fn get_context(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:context").cloned()
}

/// Set context graph pattern in Rule-IR JSON-LD
pub fn set_context(rule_jsonld: &mut Value, context: Value) -> AnyhowResult<()> {
    rule_jsonld["kotoba:context"] = context;
    Ok(())
}

/// Add a node to a graph pattern JSON-LD
pub fn add_node_to_pattern(pattern_jsonld: &mut Value, node_id: &str, node_type: Option<&str>, props: Option<Value>) -> AnyhowResult<()> {
    let nodes = pattern_jsonld.get_mut("kotoba:nodes")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:nodes must be an array")?;

    let mut node = json!({
        "kotoba:id": node_id,
    });

    if let Some(nt) = node_type {
        node["kotoba:type"] = json!(nt);
    }

    if let Some(p) = props {
        node["kotoba:props"] = p;
    }

    nodes.push(node);
    Ok(())
}

/// Add an edge to a graph pattern JSON-LD
pub fn add_edge_to_pattern(pattern_jsonld: &mut Value, edge_id: &str, src: &str, dst: &str, edge_type: Option<&str>) -> AnyhowResult<()> {
    let edges = pattern_jsonld.get_mut("kotoba:edges")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:edges must be an array")?;

    let mut edge = json!({
        "kotoba:id": edge_id,
        "kotoba:src": src,
        "kotoba:dst": dst,
    });

    if let Some(et) = edge_type {
        edge["kotoba:type"] = json!(et);
    }

    edges.push(edge);
    Ok(())
}

/// Add a type definition to Rule-IR JSON-LD
pub fn add_type_def(rule_jsonld: &mut Value, type_name: &str, labels: Vec<&str>) -> AnyhowResult<()> {
    let types = rule_jsonld.get_mut("kotoba:types")
        .and_then(|v| v.as_object_mut())
        .context("kotoba:types must be an object")?;

    types.insert(type_name.to_string(), json!(labels));
    Ok(())
}

/// Get type definitions from Rule-IR JSON-LD
pub fn get_type_defs(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:types").cloned()
}

/// Add a NAC (Negative Application Condition) to Rule-IR JSON-LD
pub fn add_nac(rule_jsonld: &mut Value, nac: Value) -> AnyhowResult<()> {
    let nacs = rule_jsonld.get_mut("kotoba:nacs")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:nacs must be an array")?;

    nacs.push(nac);
    Ok(())
}

/// Get NACs from Rule-IR JSON-LD
pub fn get_nacs(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:nacs").cloned()
}

/// Create an empty NAC JSON-LD
pub fn create_empty_nac() -> Value {
    json!({
        "kotoba:nodes": [],
        "kotoba:edges": [],
    })
}

/// Add a guard condition to Rule-IR JSON-LD
pub fn add_guard(rule_jsonld: &mut Value, guard_ref: &str, args: Value) -> AnyhowResult<()> {
    let guards = rule_jsonld.get_mut("kotoba:guards")
        .and_then(|v| v.as_array_mut())
        .context("kotoba:guards must be an array")?;

    guards.push(json!({
        "kotoba:ref": guard_ref,
        "kotoba:args": args,
    }));

    Ok(())
}

/// Get guards from Rule-IR JSON-LD
pub fn get_guards(rule_jsonld: &Value) -> Option<Value> {
    rule_jsonld.get("kotoba:guards").cloned()
}

