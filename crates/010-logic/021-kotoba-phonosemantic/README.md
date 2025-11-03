# Kotoba Phonosemantic

Phonosemantic vocabulary mapping system for Kotoba.

## Overview

This crate provides bidirectional mapping between phonemes (sound units) and semantic meanings, enabling natural language understanding through structured vocabulary relationships.

## Features

- **Phoneme Analysis**: Extract and analyze phonemes from text
- **Semantic Mapping**: Map phonemes to semantic meanings
- **Bidirectional Conversion**: phoneme → meaning and meaning → phoneme
- **Vocabulary System**: JSON-LD-based vocabulary management
- **OWL Inference**: Optional reasoning for discovering semantic relationships

## Usage

### Basic Usage

```rust
use kotoba_phonosemantic::{PhonosemanticMapper, VocabularySystem};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create vocabulary system
    let mut vocab = VocabularySystem::new();

    // Load vocabulary from JSON-LD
    let vocab_json = json!({
        "@context": {
            "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
        },
        "@graph": [
            {
                "@id": "kotoba:phoneme/a",
                "@type": "kotoba:Phoneme",
                "kotoba:phoneme": "a",
                "kotoba:semanticMeaning": "beginning, origin"
            },
            {
                "@id": "kotoba:phoneme/i",
                "@type": "kotoba:Phoneme",
                "kotoba:phoneme": "i",
                "kotoba:semanticMeaning": "movement, flow"
            }
        ]
    });

    vocab.load_from_jsonld(vocab_json).await?;

    // Create mapper
    let mapper = PhonosemanticMapper::new(vocab);

    // Map phoneme to meaning
    let meanings = mapper.phoneme_to_meaning("a").await?;
    println!("Meanings of 'a': {:?}", meanings);

    // Map meaning to phoneme
    let phonemes = mapper.meaning_to_phoneme("beginning").await?;
    println!("Phonemes for 'beginning': {:?}", phonemes);

    Ok(())
}
```

### With Confidence Scores

```rust
// Get mappings with confidence scores
let mappings = mapper.phoneme_to_meaning_with_confidence("a").await?;
for mapping in mappings {
    println!("{} -> {} (confidence: {})", 
             mapping.phoneme, mapping.meaning, mapping.confidence);
}
```

### Export Vocabulary

```rust
// Export vocabulary as JSON-LD
let jsonld = vocab.to_jsonld();
println!("{}", serde_json::to_string_pretty(&jsonld)?);
```

## Architecture

The phonosemantic system consists of:

1. **Phoneme**: Basic sound unit with phonetic features
2. **SemanticMapping**: Bidirectional mapping between phoneme and meaning
3. **VocabularySystem**: Manages all mappings in JSON-LD format
4. **PhonosemanticMapper**: Provides mapping API
5. **PhonemeAnalyzer**: Extracts phonemes from text

## JSON-LD Format

Phonemes and mappings are represented in JSON-LD:

```json
{
  "@context": {
    "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
  },
  "@graph": [
    {
      "@id": "kotoba:phoneme/a",
      "@type": "kotoba:Phoneme",
      "kotoba:phoneme": "a",
      "kotoba:semanticMeaning": "beginning, origin",
      "kotoba:phoneticFeatures": {
        "kotoba:place": "vowel",
        "kotoba:manner": "open"
      }
    },
    {
      "@id": "kotoba:mapping/a-beginning",
      "@type": "kotoba:SemanticMapping",
      "kotoba:phoneme": "a",
      "kotoba:semanticMeaning": "beginning",
      "kotoba:mappingDirection": "bidirectional",
      "kotoba:confidence": 0.95
    }
  ]
}
```

## Future Enhancements

- Advanced phonetic analysis (IPA support)
- OWL inference for discovering semantic relationships
- Multi-language support
- Context-aware mapping
- Machine learning-based confidence scoring

