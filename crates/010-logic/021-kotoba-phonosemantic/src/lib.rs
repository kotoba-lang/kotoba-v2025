//! # Kotoba Phonosemantic
//!
//! Phonosemantic vocabulary mapping system for Kotoba.
//! Provides bidirectional mapping between phonemes and semantic meanings using JSON-LD and OWL inference.
//!
//! ## Features
//!
//! - **Phoneme Analysis**: Extract and analyze phonemes from text
//! - **Semantic Mapping**: Map phonemes to semantic meanings
//! - **Bidirectional Conversion**: phoneme → meaning and meaning → phoneme
//! - **Vocabulary System**: JSON-LD-based vocabulary management
//! - **OWL Inference**: Optional reasoning for discovering semantic relationships
//!
//! ## Example
//!
//! ```rust,no_run
//! use kotoba_phonosemantic::{PhonosemanticMapper, VocabularySystem};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create vocabulary system
//! let mut vocab = VocabularySystem::new();
//!
//! // Load vocabulary from JSON-LD
//! let vocab_json = json!({
//!     "@context": {
//!         "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
//!     },
//!     "@graph": [
//!         {
//!             "@id": "kotoba:phoneme/a",
//!             "@type": "kotoba:Phoneme",
//!             "kotoba:phoneme": "a",
//!             "kotoba:semanticMeaning": "beginning, origin"
//!         }
//!     ]
//! });
//!
//! vocab.load_from_jsonld(vocab_json).await?;
//!
//! // Create mapper
//! let mapper = PhonosemanticMapper::new(vocab);
//!
//! // Map phoneme to meaning
//! let meaning = mapper.phoneme_to_meaning("a").await?;
//! println!("Meaning of 'a': {:?}", meaning);
//!
//! // Map meaning to phoneme
//! let phonemes = mapper.meaning_to_phoneme("beginning").await?;
//! println!("Phonemes for 'beginning': {:?}", phonemes);
//! # Ok(())
//! # }
//! ```

pub mod mapper;
pub mod phoneme;
pub mod vocabulary;
pub mod analysis;

pub use mapper::PhonosemanticMapper;
pub use phoneme::{Phoneme, PhoneticFeatures};
pub use vocabulary::{VocabularySystem, SemanticMapping, MappingDirection};
pub use analysis::PhonemeAnalyzer;

/// Error types for phonosemantic operations
#[derive(Debug, thiserror::Error)]
pub enum PhonosemanticError {
    #[error("JSON-LD parsing error: {0}")]
    JsonLdParse(#[from] serde_json::Error),

    #[error("Vocabulary error: {0}")]
    VocabularyError(String),

    #[error("Mapping error: {0}")]
    MappingError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, PhonosemanticError>;

