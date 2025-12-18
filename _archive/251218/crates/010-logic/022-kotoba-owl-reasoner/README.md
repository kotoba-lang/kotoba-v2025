# Kotoba OWL Reasoner

OWL reasoning engine integration for Kotoba using fukurow.

## Overview

This crate provides OWL reasoning capabilities for Kotoba's phonosemantic digital computing system:

- **RDFS Reasoning**: Transitive closure for subClassOf and subPropertyOf
- **OWL Lite Reasoning**: Tableau algorithm for class hierarchy inference
- **OWL DL Reasoning**: Complete OWL DL reasoning with all constructors
- **SHACL Validation**: Shape constraint validation
- **SPARQL Queries**: SPARQL 1.1 query execution
- **JSON-LD Integration**: Seamless JSON-LD input/output

## Usage

```rust
use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create reasoning engine with OWL DL level
    let mut engine = ReasoningEngine::new(ReasoningLevel::OwlDl)?;

    // Load ontology from JSON-LD
    let ontology_json = json!({
        "@context": {
            "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
            "owl": "http://www.w3.org/2002/07/owl#",
            "ex": "http://example.org/"
        },
        "@graph": [
            {
                "@id": "ex:Person",
                "@type": "owl:Class"
            },
            {
                "@id": "ex:Student",
                "@type": "owl:Class",
                "rdfs:subClassOf": "ex:Person"
            }
        ]
    });

    engine.load_ontology_from_jsonld(ontology_json).await?;

    // Perform reasoning
    let result = engine.reason().await?;

    // Get inferred triples as JSON-LD
    let inferred_jsonld = engine.inferred_triples_as_jsonld().await?;
    println!("Inferred triples: {}", serde_json::to_string_pretty(&inferred_jsonld)?);

    Ok(())
}
```

## Reasoning Levels

### RDFS

Basic RDFS reasoning:
- `rdfs:subClassOf` transitive closure
- `rdfs:subPropertyOf` transitive closure
- `rdfs:domain` and `rdfs:range` type inference
- `rdf:type` inference and hierarchical type propagation

### OWL Lite

OWL Lite reasoning using tableau algorithm:
- Class hierarchy inference (subsumption reasoning)
- Ontology consistency checking
- Soundness and termination guaranteed

### OWL DL

Complete OWL DL reasoning:
- Extended class constructors (intersectionOf, unionOf, complementOf, oneOf)
- Property constraints (someValuesFrom, allValuesFrom, hasValue, min/max/exactCardinality)
- Individual instance verification

## Integration with fukurow

This crate wraps fukurow's reasoning engines:
- `fukurow-rdfs`: RDFS reasoning
- `fukurow-lite`: OWL Lite reasoning
- `fukurow-dl`: OWL DL reasoning
- `fukurow-shacl`: SHACL validation
- `fukurow-sparql`: SPARQL query execution

## Future Enhancements

- Complete fukurow API integration
- Reasoning chain tracking
- Consistency checking
- Performance optimizations
- Caching strategies

