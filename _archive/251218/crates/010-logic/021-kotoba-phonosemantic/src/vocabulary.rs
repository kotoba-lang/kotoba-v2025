//! Vocabulary system for managing phoneme-semantic mappings
//!
//! Provides JSON-LD-based vocabulary management with OWL inference support.

use crate::{PhonosemanticError, Result};
use crate::phoneme::Phoneme;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::info;

/// Direction of semantic mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingDirection {
    /// Phoneme → Meaning
    PhonemeToMeaning,
    /// Meaning → Phoneme
    MeaningToPhoneme,
    /// Bidirectional
    Bidirectional,
}

/// Semantic mapping between phoneme and meaning
#[derive(Debug, Clone)]
pub struct SemanticMapping {
    /// Phoneme symbol
    pub phoneme: String,
    /// Semantic meaning
    pub meaning: String,
    /// Mapping direction
    pub direction: MappingDirection,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Vocabulary system for managing phoneme-semantic mappings
pub struct VocabularySystem {
    /// Phoneme → Meanings mapping
    phoneme_to_meanings: HashMap<String, Vec<SemanticMapping>>,
    
    /// Meaning → Phonemes mapping
    meaning_to_phonemes: HashMap<String, Vec<SemanticMapping>>,
    
    /// Phoneme registry
    phonemes: HashMap<String, Phoneme>,
}

impl VocabularySystem {
    /// Create a new vocabulary system
    pub fn new() -> Self {
        Self {
            phoneme_to_meanings: HashMap::new(),
            meaning_to_phonemes: HashMap::new(),
            phonemes: HashMap::new(),
        }
    }

    /// Load vocabulary from JSON-LD
    pub async fn load_from_jsonld(&mut self, jsonld: Value) -> Result<()> {
        info!("[VocabularySystem] Loading vocabulary from JSON-LD");

        // Extract @graph from JSON-LD
        if let Some(graph) = jsonld.get("@graph").and_then(|g| g.as_array()) {
            for node in graph {
                if let Some(node_obj) = node.as_object() {
                    // Check if this is a Phoneme node
                    if let Some(type_val) = node_obj.get("@type") {
                        if let Some(type_str) = type_val.as_str() {
                            if type_str == "kotoba:Phoneme" || type_str.contains("Phoneme") {
                                self.load_phoneme_node(node_obj)?;
                            } else if type_str == "kotoba:SemanticMapping" || type_str.contains("SemanticMapping") {
                                self.load_mapping_node(node_obj)?;
                            }
                        }
                    }
                }
            }
        }

        info!("[VocabularySystem] Loaded {} phonemes, {} mappings",
              self.phonemes.len(),
              self.phoneme_to_meanings.values().map(|v| v.len()).sum::<usize>());

        Ok(())
    }

    /// Load a phoneme node from JSON-LD
    fn load_phoneme_node(&mut self, node: &serde_json::Map<String, Value>) -> Result<()> {
        let id = node.get("@id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PhonosemanticError::VocabularyError("Missing @id".to_string()))?;

        let phoneme_symbol = node.get("kotoba:phoneme")
            .or_else(|| node.get("phoneme"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| PhonosemanticError::VocabularyError(format!("Missing phoneme symbol for {}", id)))?;

        let semantic_meaning = node.get("kotoba:semanticMeaning")
            .or_else(|| node.get("semanticMeaning"))
            .and_then(|v| v.as_str());

        // Create phoneme
        let mut phoneme = Phoneme::new(phoneme_symbol);
        if let Some(meaning) = semantic_meaning {
            phoneme.semantic_meanings.push(meaning.to_string());
        }

        // Store phoneme
        self.phonemes.insert(phoneme_symbol.to_string(), phoneme.clone());

        // Create mapping if meaning exists
        if let Some(meaning) = semantic_meaning {
            let mapping = SemanticMapping {
                phoneme: phoneme_symbol.to_string(),
                meaning: meaning.to_string(),
                direction: MappingDirection::Bidirectional,
                confidence: 1.0,
                metadata: HashMap::new(),
            };

            self.phoneme_to_meanings
                .entry(phoneme_symbol.to_string())
                .or_insert_with(Vec::new)
                .push(mapping.clone());

            self.meaning_to_phonemes
                .entry(meaning.to_string())
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        Ok(())
    }

    /// Load a semantic mapping node from JSON-LD
    fn load_mapping_node(&mut self, node: &serde_json::Map<String, Value>) -> Result<()> {
        let phoneme = node.get("kotoba:phoneme")
            .or_else(|| node.get("phoneme"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| PhonosemanticError::VocabularyError("Missing phoneme in mapping".to_string()))?;

        let meaning = node.get("kotoba:semanticMeaning")
            .or_else(|| node.get("semanticMeaning"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| PhonosemanticError::VocabularyError("Missing semanticMeaning in mapping".to_string()))?;

        let direction_str = node.get("kotoba:mappingDirection")
            .or_else(|| node.get("mappingDirection"))
            .and_then(|v| v.as_str())
            .unwrap_or("bidirectional");

        let direction = match direction_str {
            "phoneme-to-meaning" => MappingDirection::PhonemeToMeaning,
            "meaning-to-phoneme" => MappingDirection::MeaningToPhoneme,
            _ => MappingDirection::Bidirectional,
        };

        let confidence = node.get("kotoba:confidence")
            .or_else(|| node.get("confidence"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let mapping = SemanticMapping {
            phoneme: phoneme.to_string(),
            meaning: meaning.to_string(),
            direction,
            confidence,
            metadata: HashMap::new(),
        };

        if matches!(direction, MappingDirection::PhonemeToMeaning | MappingDirection::Bidirectional) {
            self.phoneme_to_meanings
                .entry(phoneme.to_string())
                .or_insert_with(Vec::new)
                .push(mapping.clone());
        }

        if matches!(direction, MappingDirection::MeaningToPhoneme | MappingDirection::Bidirectional) {
            self.meaning_to_phonemes
                .entry(meaning.to_string())
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        Ok(())
    }

    /// Get meanings for a phoneme
    pub fn get_meanings(&self, phoneme: &str) -> Vec<&SemanticMapping> {
        self.phoneme_to_meanings
            .get(phoneme)
            .map(|mappings| mappings.iter().collect())
            .unwrap_or_default()
    }

    /// Get phonemes for a meaning
    pub fn get_phonemes(&self, meaning: &str) -> Vec<&SemanticMapping> {
        self.meaning_to_phonemes
            .get(meaning)
            .map(|mappings| mappings.iter().collect())
            .unwrap_or_default()
    }

    /// Get phoneme by symbol
    pub fn get_phoneme(&self, symbol: &str) -> Option<&Phoneme> {
        self.phonemes.get(symbol)
    }

    /// Add a mapping
    pub fn add_mapping(&mut self, mapping: SemanticMapping) {
        if matches!(mapping.direction, MappingDirection::PhonemeToMeaning | MappingDirection::Bidirectional) {
            self.phoneme_to_meanings
                .entry(mapping.phoneme.clone())
                .or_insert_with(Vec::new)
                .push(mapping.clone());
        }

        if matches!(mapping.direction, MappingDirection::MeaningToPhoneme | MappingDirection::Bidirectional) {
            self.meaning_to_phonemes
                .entry(mapping.meaning.clone())
                .or_insert_with(Vec::new)
                .push(mapping);
        }
    }

    /// Export vocabulary as JSON-LD
    pub fn to_jsonld(&self) -> Value {
        let mut graph = Vec::new();

        // Export phonemes
        for (symbol, phoneme) in &self.phonemes {
            let mut node = serde_json::Map::new();
            node.insert("@id".to_string(), Value::String(format!("kotoba:phoneme/{}", symbol)));
            node.insert("@type".to_string(), Value::String("kotoba:Phoneme".to_string()));
            node.insert("kotoba:phoneme".to_string(), Value::String(symbol.clone()));

            if !phoneme.semantic_meanings.is_empty() {
                node.insert("kotoba:semanticMeaning".to_string(),
                           Value::String(phoneme.semantic_meanings[0].clone()));
            }

            graph.push(Value::Object(node));
        }

        // Export mappings
        for mappings in self.phoneme_to_meanings.values() {
            for mapping in mappings {
                let mut node = serde_json::Map::new();
                node.insert("@id".to_string(), Value::String(format!("kotoba:mapping/{}-{}",
                                                                      mapping.phoneme, mapping.meaning)));
                node.insert("@type".to_string(), Value::String("kotoba:SemanticMapping".to_string()));
                node.insert("kotoba:phoneme".to_string(), Value::String(mapping.phoneme.clone()));
                node.insert("kotoba:semanticMeaning".to_string(), Value::String(mapping.meaning.clone()));
                node.insert("kotoba:mappingDirection".to_string(),
                           Value::String(match mapping.direction {
                               MappingDirection::PhonemeToMeaning => "phoneme-to-meaning",
                               MappingDirection::MeaningToPhoneme => "meaning-to-phoneme",
                               MappingDirection::Bidirectional => "bidirectional",
                           }));
                node.insert("kotoba:confidence".to_string(), Value::Number(
                    serde_json::Number::from_f64(mapping.confidence).unwrap_or(serde_json::Number::from(1))
                ));

                graph.push(Value::Object(node));
            }
        }

        serde_json::json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
            },
            "@graph": graph
        })
    }
}

impl Default for VocabularySystem {
    fn default() -> Self {
        Self::new()
    }
}

