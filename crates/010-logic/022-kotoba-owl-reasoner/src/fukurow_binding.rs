//! Fukurow store binding for Kotoba
//!
//! Provides conversion between JSON-LD and fukurow's RDF store format.

use crate::Result;
use fukurow_core::model::Triple;
use fukurow_store::store::{RdfStore, GraphId, Provenance};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Fukurow RDF store wrapper
pub struct FukurowStore {
    /// Internal fukurow RDF store
    store: Arc<Mutex<RdfStore>>,
}

impl FukurowStore {
    /// Create a new fukurow store
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(RdfStore::new())),
        }
    }

    /// Load data from JSON-LD
    pub async fn load_from_jsonld(&mut self, jsonld: Value) -> Result<()> {
        let mut store = self.store.lock().await;
        
        // Extract @graph from JSON-LD
        if let Some(graph) = jsonld.get("@graph").and_then(|g| g.as_array()) {
            for node in graph {
                if let Some(node_obj) = node.as_object() {
                    if let Some(subject) = node_obj.get("@id").and_then(|v| v.as_str()) {
                        // Extract all properties as triples
                        for (key, value) in node_obj {
                            if key != "@id" && key != "@type" && key != "@context" {
                                let object = match value {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    Value::Bool(b) => b.to_string(),
                                    Value::Array(arr) => {
                                        // Handle arrays by creating multiple triples
                                        for item in arr {
                                            if let Some(item_str) = item.as_str() {
                                                store.insert(
                                                    Triple {
                                                        subject: subject.to_string(),
                                                        predicate: key.clone(),
                                                        object: item_str.to_string(),
                                                    },
                                                    GraphId::Default,
                                                    Provenance::System {
                                                        source: "kotoba-owl-reasoner".to_string(),
                                                    },
                                                );
                                            }
                                        }
                                        continue; // Skip the main triple for arrays
                                    }
                                    Value::Object(_) => {
                                        // Nested objects - extract @id if present
                                        if let Some(nested_id) = value.get("@id").and_then(|v| v.as_str()) {
                                            nested_id.to_string()
                                        } else {
                                            value.to_string()
                                        }
                                    }
                                    Value::Null => continue,
                                };
                                
                                store.insert(
                                    Triple {
                                        subject: subject.to_string(),
                                        predicate: key.clone(),
                                        object,
                                    },
                                    GraphId::Default,
                                    Provenance::System {
                                        source: "kotoba-owl-reasoner".to_string(),
                                    },
                                );
                            }
                        }
                        
                        // Handle @type as rdf:type
                        if let Some(type_val) = node_obj.get("@type") {
                            if let Some(type_str) = type_val.as_str() {
                                store.insert(
                                    Triple {
                                        subject: subject.to_string(),
                                        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                                        object: type_str.to_string(),
                                    },
                                    GraphId::Default,
                                    Provenance::System {
                                        source: "kotoba-owl-reasoner".to_string(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Export store data as JSON-LD
    pub async fn to_jsonld(&self) -> Result<Value> {
        let store = self.store.lock().await;
        
        // Get all triples
        let all_triples = store.find_triples(None, None, None);
        
        // Group triples by subject
        let mut subjects: std::collections::HashMap<String, serde_json::Map<String, Value>> = std::collections::HashMap::new();
        
        for stored_triple in all_triples {
            let subject = stored_triple.triple.subject.clone();
            let predicate = stored_triple.triple.predicate.clone();
            let object = stored_triple.triple.object.clone();
            
            let node = subjects.entry(subject.clone()).or_insert_with(|| {
                let mut map = serde_json::Map::new();
                map.insert("@id".to_string(), Value::String(subject));
                map
            });
            
            // Handle rdf:type specially
            if predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                node.insert("@type".to_string(), Value::String(object));
            } else {
                // Add property
                node.insert(predicate, Value::String(object));
            }
        }
        
        let graph: Vec<Value> = subjects.into_values().map(Value::Object).collect();
        
        Ok(serde_json::json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "owl": "http://www.w3.org/2002/07/owl#"
            },
            "@graph": graph
        }))
    }

    /// Get the underlying RdfStore reference (for reasoning engines)
    pub fn store(&self) -> Arc<Mutex<RdfStore>> {
        Arc::clone(&self.store)
    }

    /// Get a read-only reference to the store (for reasoning)
    /// Note: This returns a guard that must be kept alive during reasoning
    pub async fn store_guard(&self) -> tokio::sync::MutexGuard<'_, RdfStore> {
        self.store.lock().await
    }

    /// Add a triple to the store
    pub async fn add_triple(&mut self, subject: &str, predicate: &str, object: &str) -> Result<()> {
        let mut store = self.store.lock().await;
        store.insert(
            Triple {
                subject: subject.to_string(),
                predicate: predicate.to_string(),
                object: object.to_string(),
            },
            GraphId::Default,
            Provenance::System {
                source: "kotoba-owl-reasoner".to_string(),
            },
        );
        Ok(())
    }

    /// Query triples from the store
    pub async fn query_triples(
        &self,
        subject: Option<&str>,
        predicate: Option<&str>,
        object: Option<&str>,
    ) -> Result<Vec<(String, String, String)>> {
        let store = self.store.lock().await;
        let results = store.find_triples(subject, predicate, object);
        
        Ok(results
            .iter()
            .map(|stored| {
                (
                    stored.triple.subject.clone(),
                    stored.triple.predicate.clone(),
                    stored.triple.object.clone(),
                )
            })
            .collect())
    }
}

impl Default for FukurowStore {
    fn default() -> Self {
        Self::new()
    }
}
