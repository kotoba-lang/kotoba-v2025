# kotoba-ir: JSON-LD Universal Intermediate Representation

This crate provides the Intermediate Representation (IR) system for Kotoba, where all IR types are represented in JSON-LD format as the universal intermediate representation, with OWL ontology definitions and SHACL shape validation support.

## Features

- **JSON-LD Universal IR**: All IR types (Rule-IR, Query-IR, Patch-IR, Strategy-IR, Catalog-IR) are represented as JSON-LD `Value` objects
- **Direct Manipulation API**: Functions to directly manipulate IRs as JSON-LD without intermediate Rust structs
- **SHACL Validation** (optional, via `reasoning` feature): Automatic validation of IR structures against SHACL shapes
- **WASM Runtime** (optional, via `wasm` feature): Execute IRs in WebAssembly runtime

## IR Types

### Rule-IR (DPO Graph Rewriting)
- DPO (Double Pushout) typed attribute graph rewriting rules
- Supports LHS, RHS, context, NACs (Negative Application Conditions), and guards
- OWL class: `kotoba:RuleIR`

### Query-IR (GQL Logical Plan Algebra)
- GQL logical plan operators: NodeScan, Filter, Expand, Join, Project, Group, Sort, Limit, Distinct, IndexScan
- OWL class: `kotoba:QueryIR`

### Patch-IR (Differential Expressions)
- Add/Delete/Update operations for vertices and edges
- Supports property updates and edge relinking
- OWL class: `kotoba:PatchIR`

### Strategy-IR (Minimal Strategy Expressions)
- Strategy operations: once, exhaust, while, seq, choice, priority
- OWL class: `kotoba:StrategyIR`

### Catalog-IR (Schema/Index/Invariant Definitions)
- Label definitions with properties
- Index definitions
- Invariant definitions
- OWL class: `kotoba:CatalogIR`

## Usage

### Basic IR Creation and Manipulation

```rust
use kotoba_ir::*;
use serde_json::json;

// Create a Rule-IR
let mut rule = create_empty_rule_jsonld(Some("rule:test"), "my_rule");

// Set rule name
set_rule_name(&mut rule, "updated_rule")?;

// Add a node to LHS pattern
let mut lhs = get_lhs(&rule).unwrap();
add_node_to_pattern(&mut lhs, "u", Some("V"), None)?;
set_lhs(&mut rule, lhs)?;

// Add a guard condition
add_guard(&mut rule, "deg_ge", json!({"var": "u", "k": 2}))?;
```

### SHACL Validation

When the `reasoning` feature is enabled, all IR operations automatically validate against SHACL shapes:

```rust
// With reasoning feature enabled, validation happens automatically
// after every modification operation
set_rule_name(&mut rule, "new_name")?; // Validates automatically
```

Manual validation is also available:

```rust
#[cfg(feature = "reasoning")]
use kotoba_ir::validate_ir_jsonld;

let result = validate_ir_jsonld(&rule, "RuleIR").await?;
if !result.valid {
    eprintln!("Validation errors: {:?}", result.errors);
}
```

### WASM Runtime Execution

When the `wasm` feature is enabled, IRs can be executed in WebAssembly:

```rust
#[cfg(feature = "wasm")]
use kotoba_ir::{WasmRuntime, execute_rule_jsonld};

// Load WASM module and execute Rule-IR
let wasm_bytes = /* load WASM module bytes */;
let result = execute_rule_jsonld(&rule, "module_name", &wasm_bytes)?;
```

## Feature Flags

- `reasoning`: Enable SHACL validation and OWL reasoning support (requires `kotoba-owl-reasoner`)
- `wasm`: Enable WASM runtime integration (requires `wasmtime`)

## OWL Ontology

IR types are defined in OWL ontology (`schemas/ir-ontology.jsonld`):
- `kotoba:IR` (abstract base class)
- `kotoba:RuleIR` (subclass of `kotoba:IR`)
- `kotoba:QueryIR` (subclass of `kotoba:IR`)
- `kotoba:PatchIR` (subclass of `kotoba:IR`)
- `kotoba:StrategyIR` (subclass of `kotoba:IR`)
- `kotoba:CatalogIR` (subclass of `kotoba:IR`)

## SHACL Shapes

IR structures are validated against SHACL shapes:
- `schemas/ir-shapes.jsonld`: Shapes for Rule-IR, Query-IR, Patch-IR, Strategy-IR
- `schemas/catalog-shapes.jsonld`: Shapes for Catalog-IR

## JSON-LD Context

IRs use the Kotoba JSON-LD context defined in `schemas/kotoba-context.jsonld` for vocabulary expansion.

## Testing

Run tests with different feature combinations:

```bash
# Basic tests (no features)
cargo test --package kotoba-ir --lib --no-default-features

# With SHACL validation
cargo test --package kotoba-ir --lib --features reasoning

# With WASM runtime
cargo test --package kotoba-ir --lib --features wasm

# With all features
cargo test --package kotoba-ir --lib --features reasoning,wasm
```

