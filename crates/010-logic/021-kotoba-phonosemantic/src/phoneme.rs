//! Phoneme representation and analysis
//!
//! Defines phoneme structures and phonetic features.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Phoneme represents a basic sound unit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Phoneme {
    /// The phoneme symbol (e.g., "a", "i", "u")
    pub symbol: String,
    
    /// Phonetic features
    pub features: PhoneticFeatures,
    
    /// Optional semantic meaning(s)
    pub semantic_meanings: Vec<String>,
}

/// Phonetic features of a phoneme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PhoneticFeatures {
    /// Place of articulation (e.g., "bilabial", "alveolar")
    pub place: Option<String>,
    
    /// Manner of articulation (e.g., "stop", "fricative", "vowel")
    pub manner: Option<String>,
    
    /// Voicing (true = voiced, false = voiceless)
    pub voiced: Option<bool>,
    
    /// Additional features as key-value pairs
    #[serde(flatten)]
    pub additional: HashMap<String, String>,
}

impl Phoneme {
    /// Create a new phoneme
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            features: PhoneticFeatures::default(),
            semantic_meanings: Vec::new(),
        }
    }

    /// Create a phoneme with semantic meaning
    pub fn with_meaning(symbol: impl Into<String>, meaning: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            features: PhoneticFeatures::default(),
            semantic_meanings: vec![meaning.into()],
        }
    }
}

impl Default for PhoneticFeatures {
    fn default() -> Self {
        Self {
            place: None,
            manner: None,
            voiced: None,
            additional: HashMap::new(),
        }
    }
}

impl PhoneticFeatures {
    /// Create phonetic features
    pub fn new(
        place: Option<String>,
        manner: Option<String>,
        voiced: Option<bool>,
    ) -> Self {
        Self {
            place,
            manner,
            voiced,
            additional: HashMap::new(),
        }
    }
}

