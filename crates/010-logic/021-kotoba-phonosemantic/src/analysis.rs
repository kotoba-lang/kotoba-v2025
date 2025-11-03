//! Phoneme analysis from text
//!
//! Provides analysis of text to extract phonemes.

use crate::phoneme::Phoneme;
use crate::{PhonosemanticError, Result};
use std::collections::HashSet;

/// Phoneme analyzer for extracting phonemes from text
pub struct PhonemeAnalyzer {
    /// Known phonemes
    phonemes: HashSet<String>,
}

impl PhonemeAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            phonemes: HashSet::new(),
        }
    }

    /// Create analyzer with known phonemes
    pub fn with_phonemes(phonemes: Vec<String>) -> Self {
        Self {
            phonemes: phonemes.into_iter().collect(),
        }
    }

    /// Extract phonemes from text
    /// 
    /// This is a simplified implementation. In a full implementation,
    /// this would use phonetic analysis to decompose text into phonemes.
    pub fn extract_phonemes(&self, text: &str) -> Result<Vec<Phoneme>> {
        let mut result = Vec::new();
        
        // Simple character-based extraction
        // TODO: Implement proper phonetic analysis
        for ch in text.chars() {
            let symbol = ch.to_string();
            if self.phonemes.contains(&symbol) || self.phonemes.is_empty() {
                result.push(Phoneme::new(symbol));
            }
        }

        Ok(result)
    }

    /// Add known phoneme
    pub fn add_phoneme(&mut self, phoneme: impl Into<String>) {
        self.phonemes.insert(phoneme.into());
    }
}

impl Default for PhonemeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

