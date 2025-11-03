//! Phonosemantic mapper for bidirectional conversion
//!
//! Provides mapping between phonemes and semantic meanings.

use crate::vocabulary::{VocabularySystem, SemanticMapping};
use crate::{PhonosemanticError, Result};
use tracing::info;

/// Phonosemantic mapper for bidirectional conversion
pub struct PhonosemanticMapper {
    vocabulary: VocabularySystem,
}

impl PhonosemanticMapper {
    /// Create a new mapper with vocabulary system
    pub fn new(vocabulary: VocabularySystem) -> Self {
        Self { vocabulary }
    }

    /// Map phoneme to semantic meaning(s)
    pub async fn phoneme_to_meaning(&self, phoneme: &str) -> Result<Vec<String>> {
        info!("[PhonosemanticMapper] Mapping phoneme '{}' to meaning", phoneme);

        let mappings = self.vocabulary.get_meanings(phoneme);
        let meanings: Vec<String> = mappings
            .iter()
            .map(|m| m.meaning.clone())
            .collect();

        if meanings.is_empty() {
            return Err(PhonosemanticError::MappingError(
                format!("No meaning found for phoneme: {}", phoneme)
            ));
        }

        Ok(meanings)
    }

    /// Map semantic meaning to phoneme(s)
    pub async fn meaning_to_phoneme(&self, meaning: &str) -> Result<Vec<String>> {
        info!("[PhonosemanticMapper] Mapping meaning '{}' to phoneme", meaning);

        let mappings = self.vocabulary.get_phonemes(meaning);
        let phonemes: Vec<String> = mappings
            .iter()
            .map(|m| m.phoneme.clone())
            .collect();

        if phonemes.is_empty() {
            return Err(PhonosemanticError::MappingError(
                format!("No phoneme found for meaning: {}", meaning)
            ));
        }

        Ok(phonemes)
    }

    /// Get mapping with confidence scores
    pub async fn phoneme_to_meaning_with_confidence(&self, phoneme: &str) -> Result<Vec<SemanticMapping>> {
        let mappings = self.vocabulary.get_meanings(phoneme);
        Ok(mappings.into_iter().cloned().collect())
    }

    /// Get mapping with confidence scores
    pub async fn meaning_to_phoneme_with_confidence(&self, meaning: &str) -> Result<Vec<SemanticMapping>> {
        let mappings = self.vocabulary.get_phonemes(meaning);
        Ok(mappings.into_iter().cloned().collect())
    }

    /// Get vocabulary system reference
    pub fn vocabulary(&self) -> &VocabularySystem {
        &self.vocabulary
    }

    /// Get mutable vocabulary system reference
    pub fn vocabulary_mut(&mut self) -> &mut VocabularySystem {
        &mut self.vocabulary
    }
}

