//! # Kotoba JSON-LD
//!
//! JSON-LD utilities for Kotoba graph processing system.
//! Provides JSON-LD parsing, serialization, context resolution, and conversion utilities.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;
use anyhow::{Context, Result};
use thiserror::Error;

/// JSON-LD document representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdDocument {
    /// @context field
    #[serde(rename = "@context")]
    pub context: JsonLdContext,
    
    /// @id field (optional)
    #[serde(rename = "@id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// @type field (optional)
    #[serde(rename = "@type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    
    /// Other fields
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// JSON-LD context representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonLdContext {
    /// String reference to context
    String(String),
    /// Inline context object
    Object(Value),
    /// Array of contexts
    Array(Vec<JsonLdContext>),
}

/// Context resolver for fetching contexts from URLs (e.g., GitHub)
pub struct ContextResolver {
    client: reqwest::Client,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Resolve context from a URL (supports GitHub raw URLs)
    pub async fn resolve(&self, url: &str) -> Result<Value> {
        // Handle GitHub blob URLs - convert to raw URL
        let resolved_url = if url.contains("github.com") && url.contains("/blob/") {
            url.replace("/blob/", "/raw/")
        } else {
            url.to_string()
        };

        let response = self
            .client
            .get(&resolved_url)
            .send()
            .await
            .context("Failed to fetch context URL")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch context: HTTP {}", response.status());
        }

        let value: Value = response
            .json()
            .await
            .context("Failed to parse context as JSON")?;

        Ok(value)
    }

    /// Resolve context with caching (simple in-memory cache)
    pub async fn resolve_cached(&self, url: &str, cache: &mut HashMap<String, Value>) -> Result<Value> {
        if let Some(cached) = cache.get(url) {
            return Ok(cached.clone());
        }

        let resolved = self.resolve(url).await?;
        cache.insert(url.to_string(), resolved.clone());
        Ok(resolved)
    }
}

impl Default for ContextResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert JSON Schema to JSON-LD
pub fn json_schema_to_jsonld(schema: &Value, context_url: Option<&str>) -> Result<JsonLdDocument> {
    let mut doc = JsonLdDocument {
        context: if let Some(url) = context_url {
            JsonLdContext::String(url.to_string())
        } else {
            JsonLdContext::String("https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld".to_string())
        },
        id: None,
        type_: None,
        data: HashMap::new(),
    };

    if let Value::Object(obj) = schema {
        // Extract @id from $id
        if let Some(id) = obj.get("$id") {
            if let Some(id_str) = id.as_str() {
                doc.id = Some(id_str.to_string());
            }
        }

        // Extract @type from title
        if let Some(title) = obj.get("title") {
            if let Some(title_str) = title.as_str() {
                let type_name = title_str.replace(" ", "");
                doc.type_ = Some(format!("kotoba:{}", type_name));
            }
        }

        // Convert properties to JSON-LD structure
        if let Some(properties) = obj.get("properties") {
            if let Value::Object(props) = properties {
                for (key, value) in props {
                    doc.data.insert(key.clone(), value.clone());
                }
            }
        }

        // Preserve other fields
        for (key, value) in obj {
            if !["$id", "$schema", "title", "properties"].contains(&key.as_str()) {
                doc.data.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(doc)
}

/// Convert JSON-LD to JSON Schema (simplified)
pub fn jsonld_to_json_schema(doc: &JsonLdDocument) -> Result<Value> {
    let mut schema = serde_json::Map::new();

    schema.insert("$schema".to_string(), json!("https://json-schema.org/draft/2020-12/schema"));

    if let Some(id) = &doc.id {
        schema.insert("$id".to_string(), json!(id));
    }

    if let Some(type_) = &doc.type_ {
        // Extract title from @type
        let title = type_.strip_prefix("kotoba:").unwrap_or(type_);
        schema.insert("title".to_string(), json!(title));
    }

    // Convert data back to properties
    let mut properties = serde_json::Map::new();
    for (key, value) in &doc.data {
        properties.insert(key.clone(), value.clone());
    }
    if !properties.is_empty() {
        schema.insert("properties".to_string(), json!(properties));
    }

    Ok(Value::Object(schema))
}

/// Parse JSON-LD document from string
pub fn parse_jsonld(input: &str) -> Result<JsonLdDocument> {
    let value: Value = serde_json::from_str(input)
        .context("Failed to parse JSON-LD")?;

    let doc: JsonLdDocument = serde_json::from_value(value)
        .context("Failed to deserialize JSON-LD document")?;

    Ok(doc)
}

/// Serialize JSON-LD document to string
pub fn serialize_jsonld(doc: &JsonLdDocument) -> Result<String> {
    let value = serde_json::to_value(doc)
        .context("Failed to serialize JSON-LD document")?;

    let json = serde_json::to_string_pretty(&value)
        .context("Failed to format JSON-LD")?;

    Ok(json)
}

/// Expand JSON-LD document (resolve contexts and expand terms)
/// Returns the raw JSON structure preserving kotoba: prefixed keys
pub async fn expand_jsonld(doc: &JsonLdDocument, _resolver: &ContextResolver) -> Result<Value> {
    // Convert JsonLdDocument back to JSON Value, preserving all fields including kotoba: prefixed keys
    let mut result = serde_json::Map::new();

    // Add @context
    match &doc.context {
        JsonLdContext::String(url) => {
            result.insert("@context".to_string(), json!(url));
        }
        JsonLdContext::Object(obj) => {
            result.insert("@context".to_string(), obj.clone());
        }
        JsonLdContext::Array(arr) => {
            let arr_value: Vec<Value> = arr.iter().map(|ctx| match ctx {
                JsonLdContext::String(s) => json!(s),
                JsonLdContext::Object(o) => o.clone(),
                JsonLdContext::Array(_) => json!([]),
            }).collect();
            result.insert("@context".to_string(), json!(arr_value));
        }
    }

    // Add @id if present
    if let Some(id) = &doc.id {
        result.insert("@id".to_string(), json!(id));
    }

    // Add @type if present
    if let Some(type_) = &doc.type_ {
        result.insert("@type".to_string(), json!(type_));
    }

    // Add all other data fields (preserves kotoba: prefixed keys)
    for (key, value) in &doc.data {
        result.insert(key.clone(), value.clone());
    }

    Ok(Value::Object(result))
}

/// Parse JSON-LD directly from string and return as Value (simpler alternative)
pub fn parse_jsonld_to_value(input: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(input)
        .context("Failed to parse JSON-LD as JSON")?;
    Ok(value)
}

/// Extract data from JSON-LD Value, handling both prefixed and unprefixed keys
pub fn extract_jsonld_value(value: &Value, key: &str) -> Option<&Value> {
    if let Value::Object(obj) = value {
        // Try prefixed key first (kotoba:key)
        let prefixed_key = format!("kotoba:{}", key);
        if let Some(v) = obj.get(&prefixed_key) {
            return Some(v);
        }
        // Fallback to unprefixed key
        obj.get(key)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jsonld() {
        let jsonld = r#"
        {
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@id": "https://example.org/user",
            "@type": "kotoba:User",
            "name": "Alice"
        }
        "#;

        let doc = parse_jsonld(jsonld).unwrap();
        assert_eq!(doc.id, Some("https://example.org/user".to_string()));
        assert_eq!(doc.type_, Some("kotoba:User".to_string()));
    }

    #[test]
    fn test_serialize_jsonld() {
        let mut doc = JsonLdDocument {
            context: JsonLdContext::String("https://example.org/context".to_string()),
            id: Some("https://example.org/doc".to_string()),
            type_: Some("kotoba:Document".to_string()),
            data: HashMap::new(),
        };

        let json = serialize_jsonld(&doc).unwrap();
        assert!(json.contains("@context"));
        assert!(json.contains("@id"));
        assert!(json.contains("@type"));
    }
}

