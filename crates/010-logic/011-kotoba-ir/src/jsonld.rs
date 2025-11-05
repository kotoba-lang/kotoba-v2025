//! JSON-LD conversion module for Kotoba IR
//!
//! Provides conversion between Rust IR types and JSON-LD format,
//! along with SHACL validation capabilities.

use crate::{rule::*, query::*, patch::*, strategy::*};
use serde_json::{json, Value};
use std::collections::HashMap;
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Convert RuleIR to JSON-LD format
pub fn rule_ir_to_jsonld(rule: &RuleIR, id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:RuleIR",
        "kotoba:name": rule.name,
        "kotoba:lhs": graph_pattern_to_jsonld(&rule.lhs),
        "kotoba:rhs": graph_pattern_to_jsonld(&rule.rhs),
        "kotoba:context": graph_pattern_to_jsonld(&rule.context),
    });

    if let Some(rule_id) = id {
        jsonld["@id"] = json!(rule_id);
    }

    if !rule.types.is_empty() {
        let types_obj: HashMap<String, Value> = rule.types
            .iter()
            .map(|(k, v)| {
                (k.clone(), json!(v.iter().map(|l| l.to_string()).collect::<Vec<_>>()))
            })
            .collect();
        jsonld["kotoba:types"] = json!(types_obj);
    }

    if !rule.nacs.is_empty() {
        jsonld["kotoba:nacs"] = json!(rule.nacs.iter().map(|nac| {
            json!({
                "kotoba:nodes": nac.nodes.iter().map(|n| graph_element_to_jsonld(n)).collect::<Vec<_>>(),
                "kotoba:edges": nac.edges.iter().map(|e| edge_def_to_jsonld(e)).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>());
    }

    if !rule.guards.is_empty() {
        jsonld["kotoba:guards"] = json!(rule.guards.iter().map(|g| {
            json!({
                "kotoba:ref": g.ref_,
                "kotoba:args": g.args,
            })
        }).collect::<Vec<_>>());
    }

    jsonld
}

/// Convert JSON-LD to RuleIR
pub fn rule_ir_from_jsonld(jsonld: &Value) -> AnyhowResult<RuleIR> {
    let name = jsonld["kotoba:name"]
        .as_str()
        .context("Missing kotoba:name")?
        .to_string();

    let lhs = graph_pattern_from_jsonld(&jsonld["kotoba:lhs"])?;
    let rhs = graph_pattern_from_jsonld(&jsonld["kotoba:rhs"])?;
    let context = jsonld.get("kotoba:context")
        .map(|v| graph_pattern_from_jsonld(v))
        .transpose()?
        .unwrap_or_else(GraphPattern::new);

    let mut rule = RuleIR {
        name,
        types: HashMap::new(),
        lhs,
        context,
        rhs,
        nacs: Vec::new(),
        guards: Vec::new(),
    };

    if let Some(types_val) = jsonld.get("kotoba:types") {
        if let Some(types_obj) = types_val.as_object() {
            for (k, v) in types_obj {
                if let Some(labels_arr) = v.as_array() {
                    let labels: Vec<_> = labels_arr
                        .iter()
                        .filter_map(|l| l.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    rule.types.insert(k.clone(), labels);
                }
            }
        }
    }

    if let Some(nacs_arr) = jsonld.get("kotoba:nacs").and_then(|v| v.as_array()) {
        for nac_val in nacs_arr {
            let nac = RuleNac {
                nodes: nac_val.get("kotoba:nodes")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|n| graph_element_from_jsonld(n).ok()).collect())
                    .unwrap_or_default(),
                edges: nac_val.get("kotoba:edges")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|e| edge_def_from_jsonld(e).ok()).collect())
                    .unwrap_or_default(),
            };
            rule.nacs.push(nac);
        }
    }

    if let Some(guards_arr) = jsonld.get("kotoba:guards").and_then(|v| v.as_array()) {
        for guard_val in guards_arr {
            if let (Some(ref_), Some(args)) = (
                guard_val.get("kotoba:ref").and_then(|v| v.as_str()),
                guard_val.get("kotoba:args").and_then(|v| v.as_object())
            ) {
                let mut args_map = HashMap::new();
                for (k, v) in args {
                    args_map.insert(k.clone(), serde_json::from_value(v.clone())?);
                }
                rule.guards.push(Guard {
                    ref_: ref_.to_string(),
                    args: args_map,
                });
            }
        }
    }

    Ok(rule)
}

/// Convert QueryIR to JSON-LD format
pub fn query_ir_to_jsonld(query: &LogicalOp, id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:QueryIR",
        "kotoba:plan": logical_op_to_jsonld(query),
    });

    if let Some(query_id) = id {
        jsonld["@id"] = json!(query_id);
    }

    jsonld
}

/// Convert JSON-LD to QueryIR (LogicalOp)
pub fn query_ir_from_jsonld(jsonld: &Value) -> AnyhowResult<LogicalOp> {
    let plan = jsonld.get("kotoba:plan")
        .context("Missing kotoba:plan")?;
    logical_op_from_jsonld(plan)
}

/// Convert PatchIR to JSON-LD format
pub fn patch_ir_to_jsonld(patch: &Patch, id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:PatchIR",
        "kotoba:adds": {
            "kotoba:vertices": patch.adds.vertices.iter().map(|v| add_vertex_to_jsonld(v)).collect::<Vec<_>>(),
            "kotoba:edges": patch.adds.edges.iter().map(|e| add_edge_to_jsonld(e)).collect::<Vec<_>>(),
        },
        "kotoba:dels": {
            "kotoba:vertices": patch.dels.vertices.iter().map(|v| json!(v)).collect::<Vec<_>>(),
            "kotoba:edges": patch.dels.edges.iter().map(|e| json!(e)).collect::<Vec<_>>(),
        },
        "kotoba:updates": {
            "kotoba:props": patch.updates.props.iter().map(|p| update_prop_to_jsonld(p)).collect::<Vec<_>>(),
            "kotoba:relinks": patch.updates.relinks.iter().map(|r| relink_to_jsonld(r)).collect::<Vec<_>>(),
        },
    });

    if let Some(patch_id) = id {
        jsonld["@id"] = json!(patch_id);
    }

    jsonld
}

/// Convert JSON-LD to PatchIR
pub fn patch_ir_from_jsonld(jsonld: &Value) -> AnyhowResult<Patch> {
    use crate::patch::*;

    let adds = jsonld.get("kotoba:adds")
        .map(|v| {
            Ok(Adds {
                vertices: v.get("kotoba:vertices")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|v| add_vertex_from_jsonld(v).ok()).collect())
                    .unwrap_or_default(),
                edges: v.get("kotoba:edges")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|e| add_edge_from_jsonld(e).ok()).collect())
                    .unwrap_or_default(),
            })
        })
        .transpose()?
        .unwrap_or_default();

    let dels = jsonld.get("kotoba:dels")
        .map(|v| {
            Ok(Dels {
                vertices: v.get("kotoba:vertices")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                edges: v.get("kotoba:edges")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
            })
        })
        .transpose()?
        .unwrap_or_default();

    let updates = jsonld.get("kotoba:updates")
        .map(|v| {
            Ok(Updates {
                props: v.get("kotoba:props")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|p| update_prop_from_jsonld(p).ok()).collect())
                    .unwrap_or_default(),
                relinks: v.get("kotoba:relinks")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|r| relink_from_jsonld(r).ok()).collect())
                    .unwrap_or_default(),
            })
        })
        .transpose()?
        .unwrap_or_default();

    Ok(Patch { adds, dels, updates })
}

/// Convert StrategyIR to JSON-LD format
pub fn strategy_ir_to_jsonld(strategy: &StrategyIR, id: Option<&str>) -> Value {
    let mut jsonld = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:StrategyIR",
        "kotoba:strategy": strategy_op_to_jsonld(&strategy.strategy),
    });

    if let Some(strategy_id) = id {
        jsonld["@id"] = json!(strategy_id);
    }

    jsonld
}

/// Convert JSON-LD to StrategyIR
pub fn strategy_ir_from_jsonld(jsonld: &Value) -> AnyhowResult<StrategyIR> {
    let strategy_val = jsonld.get("kotoba:strategy")
        .context("Missing kotoba:strategy")?;
    let strategy_op = strategy_op_from_jsonld(strategy_val)?;
    Ok(StrategyIR::new(strategy_op))
}

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
        _ => return Err(anyhow::anyhow!("Unknown IR type: {}", ir_type)),
    };

    let shape_content = fs::read_to_string(shape_path)
        .context(format!("Failed to read SHACL shape file: {}", shape_path))?;
    let shape_jsonld: Value = serde_json::from_str(&shape_content)
        .context("Failed to parse SHACL shape JSON-LD")?;

    validate_shacl(ir_jsonld, &shape_jsonld).await
        .map_err(|e| anyhow::anyhow!("SHACL validation failed: {}", e))
}

// Helper functions for converting graph patterns and other structures

fn graph_pattern_to_jsonld(pattern: &GraphPattern) -> Value {
    json!({
        "kotoba:nodes": pattern.nodes.iter().map(|n| graph_element_to_jsonld(n)).collect::<Vec<_>>(),
        "kotoba:edges": pattern.edges.iter().map(|e| edge_def_to_jsonld(e)).collect::<Vec<_>>(),
    })
}

fn graph_pattern_from_jsonld(jsonld: &Value) -> AnyhowResult<GraphPattern> {
    Ok(GraphPattern {
        nodes: jsonld.get("kotoba:nodes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|n| graph_element_from_jsonld(n).ok()).collect())
            .unwrap_or_default(),
        edges: jsonld.get("kotoba:edges")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|e| edge_def_from_jsonld(e).ok()).collect())
            .unwrap_or_default(),
    })
}

fn graph_element_to_jsonld(elem: &GraphElement) -> Value {
    let mut jsonld = json!({
        "kotoba:id": elem.id,
    });
    if let Some(ref type_) = elem.type_ {
        jsonld["kotoba:type"] = json!(type_);
    }
    if let Some(ref props) = elem.props {
        jsonld["kotoba:props"] = json!(props);
    }
    jsonld
}

fn graph_element_from_jsonld(jsonld: &Value) -> AnyhowResult<GraphElement> {
    Ok(GraphElement {
        id: jsonld.get("kotoba:id")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:id")?
            .to_string(),
        type_: jsonld.get("kotoba:type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        props: jsonld.get("kotoba:props")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
    })
}

fn edge_def_to_jsonld(edge: &EdgeDef) -> Value {
    let mut jsonld = json!({
        "kotoba:id": edge.id,
        "kotoba:src": edge.src,
        "kotoba:dst": edge.dst,
    });
    if let Some(ref type_) = edge.type_ {
        jsonld["kotoba:type"] = json!(type_);
    }
    jsonld
}

fn edge_def_from_jsonld(jsonld: &Value) -> AnyhowResult<EdgeDef> {
    Ok(EdgeDef {
        id: jsonld.get("kotoba:id")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:id")?
            .to_string(),
        src: jsonld.get("kotoba:src")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:src")?
            .to_string(),
        dst: jsonld.get("kotoba:dst")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:dst")?
            .to_string(),
        type_: jsonld.get("kotoba:type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

fn logical_op_to_jsonld(op: &LogicalOp) -> Value {
    match op {
        LogicalOp::NodeScan { label, as_, props } => {
            let mut jsonld = json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "NodeScan",
                "kotoba:label": label,
                "kotoba:as": as_,
            });
            if let Some(props) = props {
                jsonld["kotoba:props"] = json!(props);
            }
            jsonld
        }
        LogicalOp::Filter { pred, input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Filter",
                "kotoba:pred": serde_json::to_value(pred).unwrap_or(json!(null)),
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::Expand { edge, to_as, from } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Expand",
                "kotoba:edge": serde_json::to_value(edge).unwrap_or(json!(null)),
                "kotoba:toAs": to_as,
                "kotoba:from": logical_op_to_jsonld(from),
            })
        }
        LogicalOp::Join { left, right, on } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Join",
                "kotoba:left": logical_op_to_jsonld(left),
                "kotoba:right": logical_op_to_jsonld(right),
                "kotoba:on": on,
            })
        }
        LogicalOp::Project { cols, input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Project",
                "kotoba:cols": cols,
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::Group { keys, aggregations, input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Group",
                "kotoba:keys": keys,
                "kotoba:aggregations": serde_json::to_value(aggregations).unwrap_or(json!(null)),
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::Sort { keys, input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Sort",
                "kotoba:keys": serde_json::to_value(keys).unwrap_or(json!(null)),
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::Limit { count, input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Limit",
                "kotoba:count": count,
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::Distinct { input } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "Distinct",
                "kotoba:input": logical_op_to_jsonld(input),
            })
        }
        LogicalOp::IndexScan { label, as_, index, value } => {
            json!({
                "@type": "kotoba:LogicalOp",
                "kotoba:op": "IndexScan",
                "kotoba:label": label,
                "kotoba:as": as_,
                "kotoba:index": index,
                "kotoba:value": serde_json::to_value(value).unwrap_or(json!(null)),
            })
        }
    }
}

fn logical_op_from_jsonld(jsonld: &Value) -> AnyhowResult<LogicalOp> {
    let op_str = jsonld.get("kotoba:op")
        .and_then(|v| v.as_str())
        .context("Missing kotoba:op")?;

    match op_str {
        "NodeScan" => {
            Ok(LogicalOp::NodeScan {
                label: jsonld.get("kotoba:label")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:label")?
                    .to_string(),
                as_: jsonld.get("kotoba:as")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:as")?
                    .to_string(),
                props: jsonld.get("kotoba:props")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
            })
        }
        "Filter" => {
            Ok(LogicalOp::Filter {
                pred: jsonld.get("kotoba:pred")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .context("Invalid kotoba:pred")?,
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "Expand" => {
            Ok(LogicalOp::Expand {
                edge: jsonld.get("kotoba:edge")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .context("Invalid kotoba:edge")?,
                to_as: jsonld.get("kotoba:toAs")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:toAs")?
                    .to_string(),
                from: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:from").context("Missing kotoba:from")?
                )?),
            })
        }
        "Join" => {
            Ok(LogicalOp::Join {
                left: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:left").context("Missing kotoba:left")?
                )?),
                right: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:right").context("Missing kotoba:right")?
                )?),
                on: jsonld.get("kotoba:on")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
            })
        }
        "Project" => {
            Ok(LogicalOp::Project {
                cols: jsonld.get("kotoba:cols")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "Group" => {
            Ok(LogicalOp::Group {
                keys: jsonld.get("kotoba:keys")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                aggregations: jsonld.get("kotoba:aggregations")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
                    .unwrap_or_default(),
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "Sort" => {
            Ok(LogicalOp::Sort {
                keys: jsonld.get("kotoba:keys")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
                    .unwrap_or_default(),
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "Limit" => {
            Ok(LogicalOp::Limit {
                count: jsonld.get("kotoba:count")
                    .and_then(|v| v.as_u64())
                    .context("Missing or invalid kotoba:count")? as usize,
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "Distinct" => {
            Ok(LogicalOp::Distinct {
                input: Box::new(logical_op_from_jsonld(
                    jsonld.get("kotoba:input").context("Missing kotoba:input")?
                )?),
            })
        }
        "IndexScan" => {
            Ok(LogicalOp::IndexScan {
                label: jsonld.get("kotoba:label")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:label")?
                    .to_string(),
                as_: jsonld.get("kotoba:as")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:as")?
                    .to_string(),
                index: jsonld.get("kotoba:index")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:index")?
                    .to_string(),
                value: jsonld.get("kotoba:value")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .context("Invalid kotoba:value")?,
            })
        }
        _ => Err(anyhow::anyhow!("Unknown logical operator: {}", op_str)),
    }
}

fn strategy_op_to_jsonld(op: &StrategyOp) -> Value {
    match op {
        StrategyOp::Once { rule } => {
            json!({
                "kotoba:op": "once",
                "kotoba:rule": rule,
            })
        }
        StrategyOp::Exhaust { rule, order, measure } => {
            let mut jsonld = json!({
                "kotoba:op": "exhaust",
                "kotoba:rule": rule,
                "kotoba:order": format!("{:?}", order).to_lowercase(),
            });
            if let Some(measure) = measure {
                jsonld["kotoba:measure"] = json!(measure);
            }
            jsonld
        }
        StrategyOp::While { rule, pred, order } => {
            json!({
                "kotoba:op": "while",
                "kotoba:rule": rule,
                "kotoba:pred": pred,
                "kotoba:order": format!("{:?}", order).to_lowercase(),
            })
        }
        StrategyOp::Seq { strategies } => {
            json!({
                "kotoba:op": "seq",
                "kotoba:strategies": strategies.iter().map(|s| strategy_op_to_jsonld(s)).collect::<Vec<_>>(),
            })
        }
        StrategyOp::Choice { strategies } => {
            json!({
                "kotoba:op": "choice",
                "kotoba:strategies": strategies.iter().map(|s| strategy_op_to_jsonld(s)).collect::<Vec<_>>(),
            })
        }
        StrategyOp::Priority { strategies } => {
            json!({
                "kotoba:op": "priority",
                "kotoba:strategies": strategies.iter().map(|s| {
                    json!({
                        "kotoba:strategy": strategy_op_to_jsonld(&s.strategy),
                        "kotoba:priority": s.priority,
                    })
                }).collect::<Vec<_>>(),
            })
        }
    }
}

fn strategy_op_from_jsonld(jsonld: &Value) -> AnyhowResult<StrategyOp> {
    let op_str = jsonld.get("kotoba:op")
        .and_then(|v| v.as_str())
        .context("Missing kotoba:op")?;

    match op_str {
        "once" => {
            Ok(StrategyOp::Once {
                rule: jsonld.get("kotoba:rule")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:rule")?
                    .to_string(),
            })
        }
        "exhaust" => {
            Ok(StrategyOp::Exhaust {
                rule: jsonld.get("kotoba:rule")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:rule")?
                    .to_string(),
                order: jsonld.get("kotoba:order")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "topdown" => Some(Order::TopDown),
                        "bottomup" => Some(Order::BottomUp),
                        "fair" => Some(Order::Fair),
                        _ => None,
                    })
                    .unwrap_or_default(),
                measure: jsonld.get("kotoba:measure")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        }
        "while" => {
            Ok(StrategyOp::While {
                rule: jsonld.get("kotoba:rule")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:rule")?
                    .to_string(),
                pred: jsonld.get("kotoba:pred")
                    .and_then(|v| v.as_str())
                    .context("Missing kotoba:pred")?
                    .to_string(),
                order: jsonld.get("kotoba:order")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "topdown" => Some(Order::TopDown),
                        "bottomup" => Some(Order::BottomUp),
                        "fair" => Some(Order::Fair),
                        _ => None,
                    })
                    .unwrap_or_default(),
            })
        }
        "seq" => {
            Ok(StrategyOp::Seq {
                strategies: jsonld.get("kotoba:strategies")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| strategy_op_from_jsonld(s).ok().map(|op| Box::new(op))).collect())
                    .unwrap_or_default(),
            })
        }
        "choice" => {
            Ok(StrategyOp::Choice {
                strategies: jsonld.get("kotoba:strategies")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| strategy_op_from_jsonld(s).ok().map(|op| Box::new(op))).collect())
                    .unwrap_or_default(),
            })
        }
        "priority" => {
            Ok(StrategyOp::Priority {
                strategies: jsonld.get("kotoba:strategies")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| {
                        Some(PrioritizedStrategy {
                            strategy: Box::new(strategy_op_from_jsonld(s.get("kotoba:strategy")?).ok()?),
                            priority: s.get("kotoba:priority")?.as_i64()? as i32,
                        })
                    }).collect())
                    .unwrap_or_default(),
            })
        }
        _ => Err(anyhow::anyhow!("Unknown strategy operator: {}", op_str)),
    }
}

// Patch helper functions

fn add_vertex_to_jsonld(v: &crate::patch::AddVertex) -> Value {
    json!({
        "kotoba:id": v.id,
        "kotoba:labels": v.labels,
        "kotoba:props": v.props,
    })
}

fn add_vertex_from_jsonld(jsonld: &Value) -> AnyhowResult<crate::patch::AddVertex> {
    Ok(crate::patch::AddVertex {
        id: jsonld.get("kotoba:id")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:id")?
            .to_string(),
        labels: jsonld.get("kotoba:labels")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|l| l.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        props: jsonld.get("kotoba:props")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
    })
}

fn add_edge_to_jsonld(e: &crate::patch::AddEdge) -> Value {
    json!({
        "kotoba:id": e.id,
        "kotoba:src": e.src,
        "kotoba:dst": e.dst,
        "kotoba:label": e.label,
        "kotoba:props": e.props,
    })
}

fn add_edge_from_jsonld(jsonld: &Value) -> AnyhowResult<crate::patch::AddEdge> {
    Ok(crate::patch::AddEdge {
        id: jsonld.get("kotoba:id")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:id")?
            .to_string(),
        src: jsonld.get("kotoba:src")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:src")?
            .to_string(),
        dst: jsonld.get("kotoba:dst")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:dst")?
            .to_string(),
        label: jsonld.get("kotoba:label")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:label")?
            .to_string(),
        props: jsonld.get("kotoba:props")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
    })
}

fn update_prop_to_jsonld(p: &crate::patch::UpdateProp) -> Value {
    json!({
        "kotoba:id": p.id,
        "kotoba:key": p.key,
        "kotoba:value": serde_json::to_value(&p.value).unwrap_or(json!(null)),
    })
}

fn update_prop_from_jsonld(jsonld: &Value) -> AnyhowResult<crate::patch::UpdateProp> {
    Ok(crate::patch::UpdateProp {
        id: jsonld.get("kotoba:id")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:id")?
            .to_string(),
        key: jsonld.get("kotoba:key")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:key")?
            .to_string(),
        value: jsonld.get("kotoba:value")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .context("Invalid kotoba:value")?,
    })
}

fn relink_to_jsonld(r: &crate::patch::Relink) -> Value {
    let mut jsonld = json!({
        "kotoba:edgeId": r.edge_id,
    });
    if let Some(ref new_src) = r.new_src {
        jsonld["kotoba:newSrc"] = json!(new_src);
    }
    if let Some(ref new_dst) = r.new_dst {
        jsonld["kotoba:newDst"] = json!(new_dst);
    }
    jsonld
}

fn relink_from_jsonld(jsonld: &Value) -> AnyhowResult<crate::patch::Relink> {
    Ok(crate::patch::Relink {
        edge_id: jsonld.get("kotoba:edgeId")
            .and_then(|v| v.as_str())
            .context("Missing kotoba:edgeId")?
            .to_string(),
        new_src: jsonld.get("kotoba:newSrc")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        new_dst: jsonld.get("kotoba:newDst")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

// Catalog-IR functions are now in catalog_jsonld.rs
// All Catalog-IR operations use JSON-LD directly, no Rust type conversion needed

