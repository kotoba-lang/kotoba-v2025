# KotobaOS

KotobaOS Kernel + Actor + Mediator pattern implementation for Kotoba.

## Overview

This crate implements the KotobaOS execution pattern for Kotoba's phonosemantic digital computing system:

- **Kernel**: Orchestrates process network execution
- **Actor**: Performs actions based on capabilities
- **Mediator**: Selects appropriate actors using SHACL-based reasoning
- **ProcessHandler**: Interprets and orchestrates process networks from stories
- **Provenance**: Records execution history in JSON-LD/PROV-O format
- **OWL Reasoning**: Optional integration with fukurow for semantic reasoning

## Usage

### Basic Usage

```rust
use kotoba_os::{Kernel, DefaultActor};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a story (JSON-LD format)
    let story_json = json!({
        "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
        "@graph": [
            {
                "@id": "kotoba:process/example",
                "@type": "kotoba:Process",
                "kotoba:label": "Example Process",
                "kotoba:performedBy": "kotoba:performer/actor-1"
            }
        ]
    });

    // Initialize kernel with story
    let mut kernel = Kernel::new(story_json)?;

    // Register an actor
    kernel.register_default_actor(
        "kotoba:performer/actor-1",
        "kotoba:capability/execution"
    );

    // Optionally set selection strategy
    #[cfg(feature = "reasoning")]
    kernel.mediator.set_strategy(kotoba_os::SelectionStrategy::ShaclSemantic);

    // Start orchestration
    kernel.start().await?;

    // Get provenance as JSON-LD
    let provenance = kernel.provenance_jsonld();
    println!("Provenance: {}", serde_json::to_string_pretty(&provenance)?);

    Ok(())
}
```

### With OWL Reasoning and SHACL Validation

```rust
#[cfg(feature = "reasoning")]
use kotoba_os::{Kernel, ReasoningLevel, ShaclValidator};

#[cfg(feature = "reasoning")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let story_json = json!({
        "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
        "@graph": [
            {
                "@id": "kotoba:process/example",
                "@type": "kotoba:Process",
                "kotoba:label": "Example Process",
                "kotoba:performedBy": "kotoba:performer/actor-1"
            }
        ]
    });

    // Initialize kernel with OWL reasoning enabled
    // This automatically enables SHACL validation
    let mut kernel = Kernel::with_reasoning(story_json, ReasoningLevel::OwlDl)?;

    // Enable strict SHACL validation (fail on validation errors)
    kernel.enable_strict_validation();

    // Enable SHACL-based semantic actor selection
    kernel.mediator.set_strategy(kotoba_os::SelectionStrategy::ShaclSemantic);

    kernel.register_default_actor("kotoba:performer/actor-1", "kotoba:capability/execution");
    kernel.start().await?;

    // Get inferred triples from OWL reasoning
    if let Some(inferred) = kernel.get_inferred_triples().await {
        println!("Inferred triples: {}", serde_json::to_string_pretty(&inferred)?);
    }

    Ok(())
}
```

## Architecture

The execution flow follows the KotobaOS pattern:

```
Story (JSON-LD) → Kernel → ProcessHandler → [Process] → Mediator → Actor → Provenance
                                        ↓
                                   OWL Reasoning (optional)
```

1. **Story Loading**: Story is parsed from JSON-LD format
2. **Process Extraction**: ProcessHandler extracts processes from the story graph
3. **Topological Sorting**: Processes are ordered based on `next` property links
4. **OWL Reasoning** (optional): If enabled, OWL reasoning is performed on each process
5. **SHACL Validation** (optional): If enabled, processes are validated against SHACL shapes
6. **Actor Selection**: Mediator selects appropriate actors for each process
7. **Execution**: Actors perform processes and return results
8. **Provenance Recording**: All execution history is recorded in JSON-LD/PROV-O format

## Components

### Kernel

Central orchestrator that manages the execution lifecycle.

- Loads and validates Story (JSON-LD)
- Registers actors via Mediator
- Executes processes in order
- Records provenance automatically
- Optional OWL reasoning integration
- Optional SHACL validation for Process/Resource/Performer types

### Actor

Components that perform actions based on capabilities.

- Identified by capability IRI
- Resolve I/O from SHACL shapes (future)
- Execute processes via `perform()` method
- Wrap output with provenance metadata

### Mediator

Selects appropriate actors for process execution.

- **Direct Strategy**: Simple mapping based on `performedBy` IRI
- **Capability Strategy**: Match by capability IRI
- **SHACL Semantic Strategy**: SHACL-based semantic matching with compatibility scoring
- Fallback strategies for actor selection

### ProcessHandler

Orchestrates process network execution.

- Extracts processes from Story graph
- Finds initial processes (not referenced by `next`)
- Builds execution chains following `next` properties
- Returns ordered process list

### Provenance

Records execution history in PROV-O format.

- Links processes, actors, and results
- Records timestamps and metadata
- Exports to JSON-LD format

## Features

- **JSON-LD Native**: All data structures use JSON-LD format
- **OWL Reasoning**: Optional integration with fukurow for RDFS/OWL Lite/OWL DL reasoning
- **SHACL Validation**: Optional validation of Process/Resource/Performer against SHACL shapes
- **Provenance Tracking**: Complete execution history in PROV-O format
- **Async Execution**: Built on Tokio for async/await support
- **Type Safety**: Strong typing with Rust's type system

## Future Enhancements

- Complete fukurow-shacl integration (currently placeholder)
- SHACL-based actor selection (semantic matching)
- SPARQL query compilation from shapes
- Persistent provenance storage
- Error handling and retry mechanisms
- GraphStream engine integration
- Semantic Design Loop implementation
