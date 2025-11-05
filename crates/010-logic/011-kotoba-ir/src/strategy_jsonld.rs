//! JSON-LD direct manipulation API for Strategy-IR
//!
//! Provides functions to directly manipulate Strategy-IR as JSON-LD Value objects,
//! without requiring Rust struct types.

use serde_json::{json, Value};
use anyhow::{Context, Result as AnyhowResult};

const KOTOBA_CONTEXT: &str = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld";

/// Create an empty Strategy-IR as JSON-LD
pub fn create_empty_strategy_jsonld(id: Option<&str>) -> Value {
    let mut strategy = json!({
        "@context": KOTOBA_CONTEXT,
        "@type": "kotoba:StrategyIR",
        "kotoba:strategy": null,
    });

    if let Some(strategy_id) = id {
        strategy["@id"] = json!(strategy_id);
    }

    strategy
}

/// Get strategy operator from Strategy-IR JSON-LD
pub fn get_strategy(strategy_jsonld: &Value) -> Option<Value> {
    strategy_jsonld.get("kotoba:strategy").cloned()
}

/// Set strategy operator in Strategy-IR JSON-LD
pub fn set_strategy(strategy_jsonld: &mut Value, strategy: Value) -> AnyhowResult<()> {
    strategy_jsonld["kotoba:strategy"] = strategy;
    Ok(())
}

/// Create a Once strategy operator JSON-LD
pub fn create_once(rule: &str) -> Value {
    json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "Once",
        "kotoba:rule": rule,
    })
}

/// Create an Exhaust strategy operator JSON-LD
pub fn create_exhaust(rule: &str, order: &str, measure: Option<&str>) -> Value {
    let mut op = json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "Exhaust",
        "kotoba:rule": rule,
        "kotoba:order": order,
    });

    if let Some(m) = measure {
        op["kotoba:measure"] = json!(m);
    }

    op
}

/// Create a While strategy operator JSON-LD
pub fn create_while(rule: &str, pred: &str, order: &str) -> Value {
    json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "While",
        "kotoba:rule": rule,
        "kotoba:pred": pred,
        "kotoba:order": order,
    })
}

/// Create a Seq strategy operator JSON-LD
pub fn create_seq(strategies: Vec<Value>) -> Value {
    json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "Seq",
        "kotoba:strategies": strategies,
    })
}

/// Create a Choice strategy operator JSON-LD
pub fn create_choice(strategies: Vec<Value>) -> Value {
    json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "Choice",
        "kotoba:strategies": strategies,
    })
}

/// Create a Priority strategy operator JSON-LD
pub fn create_priority(strategies: Vec<Value>) -> Value {
    json!({
        "@type": "kotoba:StrategyOp",
        "kotoba:op": "Priority",
        "kotoba:strategies": strategies,
    })
}

/// Get operator type from strategy operator JSON-LD
pub fn get_operator_type(op_jsonld: &Value) -> Option<String> {
    op_jsonld.get("kotoba:op")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

