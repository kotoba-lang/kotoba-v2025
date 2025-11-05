---
layout: default
title: Architecture Overview
---

# Kotoba Architecture: Phonosemantic Digital Computing System

This document describes the comprehensive architecture of Kotoba, a **phonosemantic digital computing system** that integrates semantic reasoning, OWL inference, and self-evolution mechanisms through JSON-LD and the **Process Network Graph Model**.

## Overview

Kotoba represents a convergence of phonosemantics, semantic web technologies, and distributed systems, offering a unified framework for semantically-driven computation. The system combines:

- **Phonosemantic Vocabulary System**: Systematic mapping between phonemes and semantic meanings
- **OWL Inference Engine**: Complete reasoning capabilities (RDFS + OWL Lite + OWL DL) powered by [fukurow](https://github.com/com-junkawasaki/fukurow)
- **Semantic Execution Pattern**: Kernel + Actor + Mediator architecture inspired by [semanticos](https://github.com/com-junkawasaki/semanticos)
- **JSON-LD Native**: All computing layers use JSON-LD for representation
- **Self-Evolution**: Semantic Design Loop for continuous system improvement
- **Process Network Graph Model**: Declarative configuration management with automatic dependency resolution
- **MVCC + Merkle DAG Persistence**: Consistent distributed data management
- **Content-Addressed Storage**: CID-based addressing for location independence

## Theoretical Foundations

### Mathematical Formalization

#### Graph Theory Foundation
Kotoba implements Double Pushout (DPO) graph rewriting with complete mathematical formalization:

**Graph Definition:**
```math
G = (V, E, s, t, λ_V, λ_E)
```
Where V represents vertices, E represents edges, and λ provides labeling functions.

**DPO Production:**
```math
p = (L ← K → R)
```
Complete implementation with formal mathematical foundation.

#### Process Network Graph Model
The core architectural framework is formally defined as:

**Process Network Graph:**
```math
PNG = (P, C, λ_P, λ_C, τ)
```
- P: Set of process nodes
- C: Set of communication channels
- λ_P: Process function mapping
- λ_C: Data type mapping
- τ: Dependency relation

**Topological Execution Theorem:**
```math
∀p_i, p_j ∈ P: (τ(p_i, p_j) = 1) ⟹ π(p_i) < π(p_j)
```

### Core Principles

#### 1. Process Network Architecture
All system components are centrally managed through `dag.jsonnet` with automatic topological sorting.

**Key Properties:**
- **Termination**: Well-formed process networks guarantee termination
- **Deadlock Freedom**: Acyclic communication patterns with bounded buffers
- **Consistency Preservation**: Graph rewriting operations maintain structural integrity

#### 2. Declarative Programming Paradigm
Users define graph structures, rewriting rules, and execution strategies without imperative code.

**Design Decisions:**
- **Jsonnet Format**: Complete 0.21.0 implementation with 38/38 test compatibility
- **.kotoba Files**: Declarative configuration for graph processing
- **Automatic Optimization**: Query planning and execution optimization

#### 3. Distributed Execution with Merkle DAG
MVCC + Merkle DAG persistence with CID-based addressing ensures location-independent references.

**Benefits:**
- **Consistency**: MVCC guarantees across distributed nodes
- **Integrity**: Cryptographic hashing prevents data corruption
- **Scalability**: Content-addressed storage enables efficient distribution
- **Versioning**: Merkle DAG provides complete version history

### 4. Phonosemantic Vocabulary System

Kotoba implements a systematic mapping between phonemes (sound units) and semantic meanings, enabling natural language understanding through structured vocabulary relationships.

**Key Concepts:**
- **Phoneme**: Basic sound unit with phonetic features
- **Semantic Mapping**: Bidirectional conversion between phonemes and meanings
- **Vocabulary System**: JSON-LD-based vocabulary management with OWL inference
- **Natural Language Integration**: Structured relationships enable semantic understanding

**Mathematical Formalization:**
```math
V = (P, S, M)
```
Where:
- P: Set of phonemes
- S: Set of semantic meanings
- M: Mapping function M: P → 2^S (phoneme to set of meanings)

**Bidirectional Mapping:**
```math
M⁻¹: S → 2^P (meaning to set of phonemes)
```

### 5. OWL Inference Integration

Kotoba integrates complete OWL reasoning capabilities through the fukurow engine, enabling logical deduction and knowledge discovery.

**Inference Layers:**

1. **RDFS Inference**
   - Transitive closure of `rdfs:subClassOf`
   - Transitive closure of `rdfs:subPropertyOf`
   - Domain and range inference
   - Type inference through `rdf:type`

2. **OWL Lite Inference**
   - Tableau algorithm for consistency checking
   - Subsumption reasoning (class hierarchy)
   - Property chain inference
   - Individual instance verification

3. **OWL DL Inference**
   - Extended tableau algorithm
   - Complex class constructors (intersectionOf, unionOf, complementOf)
   - Property constraints (someValuesFrom, allValuesFrom, cardinality)
   - Complete logical reasoning

**Integration Architecture:**
```
JSON-LD Input → fukurow RdfStore → OWL Reasoner → Inferred Triples → JSON-LD Output
```

### 6. JSON-LD Universal Intermediate Representation (IR)

Kotoba uses JSON-LD as the universal format for all Intermediate Representations, enabling semantic interoperability across all IR types.

**IR Types:**

1. **Rule-IR (DPO Graph Rewriting)**
   - Represented as JSON-LD with OWL class `kotoba:RuleIR`
   - SHACL shape validation for rule structure
   - Supports LHS, RHS, context, NACs, and guards

2. **Query-IR (GQL Logical Plan Algebra)**
   - Represented as JSON-LD with OWL class `kotoba:QueryIR`
   - Logical operations (NodeScan, Filter, Expand, Join, Project, etc.)
   - SHACL validation for query plan structure

3. **Patch-IR (Differential Expressions)**
   - Represented as JSON-LD with OWL class `kotoba:PatchIR`
   - Add/Delete/Update operations for vertices and edges
   - SHACL validation for patch structure

4. **Strategy-IR (Minimal Strategy Expressions)**
   - Represented as JSON-LD with OWL class `kotoba:StrategyIR`
   - Strategy operations (once, exhaust, while, seq, choice, priority)
   - SHACL validation for strategy structure

**SHACL Validation:**
- All IR operations automatically validate against SHACL shapes when `reasoning` feature is enabled
- Validation occurs after every modification operation (set, add, delete, update)
- Invalid IR structures are rejected with detailed error messages
- SHACL shapes defined in `schemas/ir-shapes.jsonld` and `schemas/catalog-shapes.jsonld`

**WASM Runtime Integration:**
- JSON-LD IRs can be executed in WebAssembly runtime for high-performance execution
- Enabled via `wasm` feature flag
- Supports execution of Rule-IR, Query-IR, Patch-IR, and Strategy-IR in WASM
- WASM modules can be loaded and cached for efficient execution
- Integration with fukurow WASM engine for OWL reasoning support

**Benefits:**
- **Semantic Interoperability**: All IRs use the same JSON-LD format
- **OWL Reasoning**: IR structures can be reasoned about using OWL inference
- **SHACL Validation**: Structural integrity guaranteed through SHACL shapes (mandatory when `reasoning` feature enabled)
- **WASM Execution**: High-performance execution in WebAssembly runtime
- **Content-Addressable**: IR instances can be referenced by CID (Content ID)

### 7. Capability-Based Actor Selection

Kotoba implements an OWL-based capability system for matching actors to processes.

**Capability System:**

1. **Capability Ontology (OWL)**
   - Base class: `kotoba:Capability`
   - Subclasses: `kotoba:ComputeCapability`, `kotoba:StorageCapability`, `kotoba:NetworkCapability`
   - Relationships: `kotoba:requiresCapability` (Process → Capability), `kotoba:providesCapability` (Actor → Capability)

2. **Capability Matching**
   - OWL reasoning for subsumption-based matching
   - Process requirements matched against actor capabilities
   - SHACL validation for capability constraints

3. **Actor Selection**
   - Mediator uses OWL reasoning to find matching actors
   - Capability registry for efficient lookup
   - Fallback to direct mapping if OWL reasoning unavailable

**Execution Flow:**
```
Process (requiresCapability) → OWL Reasoning → Capability Matching → Actor Selection → Actor Execution
```

### 8. Semantic Execution Pattern (semanticos)

Kotoba adopts the Kernel + Actor + Mediator pattern for executing process networks defined in JSON-LD.

**Components:**

1. **Kernel**
   - Orchestrates process network graph execution
   - Manages execution lifecycle (start, execute, end)
   - Records provenance information
   - Supports OWL reasoning for process execution

2. **Mediator**
   - Selects appropriate actors based on process requirements
   - Uses OWL reasoning-based capability matching
   - Supports multiple selection strategies (Direct, Capability, ShaclSemantic)

3. **Actor**
   - Performs actions based on capabilities
   - OWL capability definitions for actor capabilities
   - Resolves I/O from SHACL shapes
   - Wraps output with provenance metadata

4. **Provenance**
   - Records all execution history in JSON-LD/PROV-O format
   - Links processes, actors, and results
   - Enables self-evolution through history analysis

**Execution Flow:**
```
Process (JSON-LD) → Kernel → Mediator (OWL Capability Matching) → Actor Selection → Actor Execution → Result (JSON-LD) → Provenance Recording
```

### 7. Self-Evolution via Semantic Design Loop

Kotoba implements a self-evolution mechanism that continuously improves the system by analyzing provenance and applying OWL inference.

**Evolution Cycle:**
1. **Shape Definition**: Define SHACL shapes for processes
2. **Process Execution**: Execute processes via Kernel
3. **Provenance Collection**: Record execution history
4. **Pattern Discovery**: Use OWL inference to discover optimization patterns
5. **Shape Refinement**: Automatically improve shape definitions
6. **Iteration**: Repeat cycle for continuous improvement

**Mathematical Model:**
```math
E(S, P, Prov) = S' where S' = refine(S, infer(Prov))
```
Where:
- S: Current shape definition
- P: Process execution
- Prov: Provenance history
- S': Refined shape definition

### 8. Embedded First

KotobaDB runs embedded in the application process.

**Advantages:**
- **Zero Deployment**: No separate database process
- **Tight Integration**: Direct API access
- **Performance**: No network overhead
- **Simplicity**: Single process to manage

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Self-Evolution Layer                      │
│     (Semantic Design Loop: Shape → Process → Provenance)    │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │  Evolution Engine (OWL-based pattern discovery)         │ │
│ └─────────────────────────────────────────────────────────┘ │
└────────────────────────────┬────────────────────────────────┘
                              │ JSON-LD Provenance
┌─────────────────────────────┼────────────────────────────────┐
│              Semantic Execution Layer (semanticos)           │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │   Kernel     │   │   Mediator   │   │    Actor    │      │
│ │ (Orchestrate)│   │ (Select Actor)│   │ (Perform)   │      │
│ └──────┬───────┘   └──────┬───────┘   └──────┬──────┘      │
└────────┼───────────────────┼──────────────────┼─────────────┘
         │ JSON-LD Process   │ SHACL Reason      │ JSON-LD Result
         ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                    OWL Inference Layer (fukurow)            │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │ RDFS Reasoner│   │ OWL Lite     │   │ OWL DL      │      │
│ │ (Transitive) │   │ (Tableau)    │   │ (Extended)  │      │
│ └──────┬───────┘   └──────┬───────┘   └──────┬───────┘      │
│        └──────────────────┼──────────────────┘            │
│                            │ JSON-LD Triples                │
└────────────────────────────┼────────────────────────────────┘
                             │
┌────────────────────────────┼────────────────────────────────┐
│                   JSON-LD Data Layer                         │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │ Phonosemantic│   │  Vocabulary │   │  Process    │      │
│ │   Mapping    │   │   System    │   │  Network    │      │
│ └──────────────┘   └──────────────┘   └──────────────┘      │
└────────────────────────────┼────────────────────────────────┘
                             │
┌────────────────────────────┼────────────────────────────────┐
│              Application Layer (KotobaDB API)                │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │    Transaction Manager                                 │ │
│ │    Query Engine                                       │ │
│ │    Version Control                                    │ │
│ └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│         Storage Engine Layer                                │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │   Content-Addressed Storage (CAS) - Merkle DAG        │ │
│ └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│          Storage Backends                                   │
│ ┌─────────────┬─────────────┬─────────────────────────────┐ │
│ │   Memory    │     LSM     │        ...                 │ │
│ └─────────────┴─────────────┴─────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Data Model

### Core Entities

#### Node
```rust
struct Node {
    cid: Cid,                    // Content identifier
    properties: BTreeMap<String, Value>, // Key-value properties
}
```

#### Edge
```rust
struct Edge {
    source: Cid,                 // Source node CID
    target: Cid,                 // Target node CID
    properties: BTreeMap<String, Value>, // Relationship properties
}
```

#### Value Types
```rust
enum Value {
    String(String),     // UTF-8 text
    Int(i64),          // 64-bit integers
    Float(f64),        // 64-bit floats
    Bool(bool),        // Booleans
    Bytes(Vec<u8>),    // Binary data
    Link(Cid),         // References to other entities
}
```

### Content Addressing

All entities are identified by their CID (Content Identifier):

```rust
struct Cid([u8; 32]); // BLAKE3 hash of CBOR-serialized data
```

**CID Generation:**
1. Serialize entity to CBOR
2. Compute BLAKE3 hash
3. Use hash as identifier

**Benefits:**
- **Uniqueness**: Each CID represents unique content
- **Integrity**: Content changes result in different CID
- **Deduplication**: Same content = same CID
- **Addressing**: Direct content lookup by hash

## Storage Architecture

### Storage Engine Trait

All storage backends implement the `StorageEngine` trait:

```rust
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn delete(&mut self, key: &[u8]) -> Result<()>;
    async fn scan(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}
```

### Available Engines

#### 1. Memory Engine
- **Purpose**: Development, testing, temporary storage
- **Characteristics**: Fast, volatile, no persistence
- **Use Cases**: Unit tests, development environments

#### 2. LSM Engine
- **Purpose**: Production persistent storage
- **Characteristics**: High write throughput, efficient reads
- **Components**: MemTable, SSTable, WAL, Compaction

### Content-Addressed Storage (CAS)

The CAS layer provides high-level operations on content-addressed data:

```rust
impl StorageEngine {
    pub async fn put_block(&mut self, block: &Block) -> Result<Cid>
    pub async fn get_block(&self, cid: &Cid) -> Result<Option<Block>>
}
```

**CAS Operations:**
- **Put**: Store block, return CID
- **Get**: Retrieve block by CID
- **Delete**: Remove block by CID
- **Exists**: Check if CID exists

## Transaction System

### ACID Properties

KotobaDB provides full ACID guarantees:

- **Atomicity**: All operations in a transaction succeed or fail together
- **Consistency**: Database remains in valid state
- **Isolation**: Concurrent transactions don't interfere
- **Durability**: Committed changes persist

### Transaction Implementation

```rust
struct Transaction {
    id: u64,
    operations: Vec<Operation>,
    status: TransactionStatus,
}

enum Operation {
    CreateNode { properties: BTreeMap<String, Value> },
    UpdateNode { cid: Cid, properties: BTreeMap<String, Value> },
    DeleteNode { cid: Cid },
    CreateEdge { source: Cid, target: Cid, properties: BTreeMap<String, Value> },
    UpdateEdge { cid: Cid, properties: BTreeMap<String, Value> },
    DeleteEdge { cid: Cid },
}
```

### MVCC (Multi-Version Concurrency Control)

- **Versioning**: Each entity has multiple versions
- **Isolation**: Readers see consistent snapshots
- **Conflicts**: Write-write conflicts detected and resolved
- **Performance**: No read locks during writes

## Version Control System

### Merkle DAG

Data history forms a Merkle DAG:

```
A ── B ── C ── D (main)
     │         │
     └─ E ── F (feature)
         │
         └─ G (bugfix)
```

**Properties:**
- **Immutable**: History cannot be changed
- **Verifiable**: Each commit links to previous
- **Mergeable**: Multiple branches can be combined

### Branching Model

```rust
struct Branch {
    name: String,
    head: Cid,          // Current HEAD commit
    base: Option<Cid>,  // Branch point
}

struct Commit {
    cid: Cid,
    parent: Option<Cid>,
    changes: Vec<Operation>,
    timestamp: u64,
    message: String,
}
```

### Operations

#### Branch Creation
```rust
let branch_id = db.create_branch("feature/x", "main").await?;
```

#### Checkout
```rust
db.checkout_branch("feature/x").await?;
```

#### Commit
```rust
let commit_id = db.commit("Implement feature X").await?;
```

#### Merge
```rust
db.merge_branch("feature/x", "main").await?;
```

## Query System

### Query Model

Queries are property-based with graph traversal:

```rust
// Find nodes by properties
let users = db.find_nodes(&[
    ("type".to_string(), Value::String("user".to_string())),
    ("active".to_string(), Value::Bool(true)),
]).await?;

// Traverse relationships
let friends = db.traverse(user_cid, |node, depth| {
    depth < 3 && node.properties.get("type") == Some(&Value::String("person".to_string()))
}).await?;
```

### Query Execution

1. **Parse**: Convert query to execution plan
2. **Index Lookup**: Use property indexes for efficient access
3. **Traversal**: Follow relationships in graph
4. **Filter**: Apply predicates and limits
5. **Return**: Stream results to caller

### Indexing Strategy

- **Property Indexes**: Automatic indexing of frequently queried properties
- **CID Indexes**: Direct lookup by content identifier
- **Edge Indexes**: Efficient relationship traversal
- **Composite Indexes**: Multi-property query optimization

## API Design

### High-Level API

The main `DB` struct provides the user interface:

```rust
pub struct DB {
    engine: Box<dyn StorageEngine>,
    // Internal state...
}

impl DB {
    // Node operations
    pub async fn create_node(&self, properties: BTreeMap<String, Value>) -> Result<Cid>
    pub async fn get_node(&self, cid: Cid) -> Result<Option<Node>>
    pub async fn update_node(&self, cid: Cid, properties: BTreeMap<String, Value>) -> Result<()>
    pub async fn delete_node(&self, cid: Cid) -> Result<()>

    // Edge operations
    pub async fn create_edge(&self, source: Cid, target: Cid, properties: BTreeMap<String, Value>) -> Result<Cid>
    pub async fn get_edge(&self, cid: Cid) -> Result<Option<Edge>>
    pub async fn find_edges(&self, filters: &[(&str, Value)]) -> Result<Vec<(Cid, Edge)>>

    // Query operations
    pub async fn find_nodes(&self, filters: &[(&str, Value)]) -> Result<Vec<(Cid, Node)>>
    pub async fn traverse(&self, start: Cid, predicate: impl Fn(&Node, usize) -> bool) -> Result<Vec<Cid>>

    // Transaction operations
    pub async fn begin_transaction(&self) -> Result<u64>
    pub async fn add_operation(&self, txn_id: u64, op: Operation) -> Result<()>
    pub async fn commit_transaction(&self, txn_id: u64) -> Result<()>
    pub async fn rollback_transaction(&self, txn_id: u64) -> Result<()>

    // Version control
    pub async fn create_branch(&self, name: &str, base: &str) -> Result<String>
    pub async fn checkout_branch(&self, name: &str) -> Result<()>
    pub async fn create_snapshot(&self, name: &str) -> Result<Snapshot>
    pub async fn restore_snapshot(&self, name: &str) -> Result<()>
}
```

### Builder Pattern

Configuration uses the builder pattern:

```rust
let db = DB::builder()
    .engine(LSMStorageEngine::new("./data").await?)
    .cache_size(100 * 1024 * 1024)  // 100MB cache
    .max_connections(10)
    .build()
    .await?;
```

## Serialization Format

### CBOR (Concise Binary Object Representation)

All data is serialized using CBOR:

```rust
// Node serialization
{
    "properties": {
        "name": "Alice",
        "age": 30,
        "active": true
    }
}

// Edge serialization
{
    "source": h'0123456789abcdef...',  // CID as bytes
    "target": h'fedcba9876543210...',
    "properties": {
        "type": "friend",
        "since": "2024"
    }
}
```

**CBOR Benefits:**
- **Compact**: Efficient binary format
- **Standard**: RFC 7049 compliant
- **Interoperable**: Works across languages
- **Extensible**: Supports custom types

## Error Handling

### Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum KotobaError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Version control error: {0}")]
    VersionControl(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("CID error: {0}")]
    Cid(String),
}
```

### Error Propagation

Errors bubble up through the stack with context:

```rust
async fn complex_operation(&self) -> Result<()> {
    let node = self.get_node(cid)
        .await
        .map_err(|e| anyhow!("Failed to get node {}: {}", cid, e))?;

    // ... more operations that can fail
}
```

## Concurrency Model

### Async/Await

All operations are async for non-blocking I/O:

```rust
// Concurrent operations
let (users, products) = tokio::try_join!(
    db.find_nodes(&[("type", Value::String("user"))]),
    db.find_nodes(&[("type", Value::String("product"))])
)?;
```

### Internal Synchronization

- **RwLock**: Reader-writer locks for shared state
- **Channels**: Async communication between components
- **Atomic Operations**: Lock-free counters and flags

## Testing Strategy

### Unit Tests
- **Core Types**: CID, Value, Block serialization
- **Storage Engines**: Individual backend testing
- **Query Engine**: Query execution testing

### Integration Tests
- **Full API**: End-to-end operation testing
- **Concurrency**: Multi-threaded access testing
- **Persistence**: Crash recovery testing

### Property-Based Testing
```rust
proptest! {
    #[test]
    fn cid_roundtrip(value in any::<Value>()) {
        let block = Block::Node(Node { properties: btreemap!{"value" => value} });
        let cid = block.cid()?;
        let serialized = block.to_bytes()?;
        let deserialized = Block::from_bytes(&serialized)?;
        prop_assert_eq!(block, deserialized);
    }
}
```

## Performance Considerations

### Memory Management
- **Object Pool**: Reuse frequently allocated objects
- **Streaming**: Process large datasets without full loading
- **GC Pressure**: Minimize allocations in hot paths

### I/O Optimization
- **Buffering**: Batch I/O operations
- **Prefetching**: Anticipatory data loading
- **Compression**: Optional transparent compression

### CPU Optimization
- **SIMD**: Vectorized operations where possible
- **Inlining**: Critical path function inlining
- **Cache-Friendly**: Data structures optimized for cache

## Extensibility

### Custom Storage Engines

Implement `StorageEngine` for custom backends:

```rust
pub struct MyStorageEngine {
    // Custom implementation
}

#[async_trait]
impl StorageEngine for MyStorageEngine {
    // Implement required methods
}
```

### Custom Value Types

Extend the `Value` enum for domain-specific types:

```rust
#[derive(Serialize, Deserialize)]
enum CustomValue {
    Standard(Value),
    DomainSpecific(MyType),
}
```

### Plugin System

Future plugin support for:
- **Custom Indexes**: Specialized indexing strategies
- **Query Extensions**: Domain-specific query languages
- **Storage Plugins**: Additional storage backends

## Security Model

### Data Integrity
- **Cryptographic Hashes**: SHA-256 or BLAKE3 for CIDs
- **Merkle Trees**: Verify data integrity
- **Digital Signatures**: Optional signed commits

### Access Control
- **Permission Model**: Future user/role-based access
- **Encryption**: Optional at-rest encryption
- **Audit Logging**: Complete operation history

## Deployment Models

### Embedded
```rust
// Direct embedding in application
let db = DB::open_lsm("./local.db").await?;
```

### Distributed
```rust
// Future: Distributed deployment
let cluster = KotobaCluster::new(vec![
    "node1:8080".to_string(),
    "node2:8080".to_string(),
    "node3:8080".to_string(),
]).await?;
```

### Cloud-Native
```rust
// Future: Cloud storage integration
let db = DB::open_cloud("s3://my-bucket/db").await?;
```

## Future Roadmap

### Short Term
- **Query Language**: SQL-like graph query language
- **Secondary Indexes**: Additional indexing options
- **Backup/Restore**: Automated backup utilities

### Medium Term
- **Distributed Operation**: Multi-node clusters
- **Graph Algorithms**: Built-in graph analytics
- **Plugin System**: Extensible architecture

### Long Term
- **Multi-Model**: Support for documents, time-series
- **Federation**: Cross-database queries
- **Global Scale**: Planet-scale deployments

## Summary

KotobaDB's architecture combines proven database techniques with modern distributed systems principles:

- **Graph-Native**: Optimized for connected data
- **Version Control**: Git-like semantics for data
- **Content Addressing**: Immutable, verifiable data storage
- **ACID Transactions**: Reliable data operations
- **Embedded Design**: Simple deployment and management

The layered architecture provides flexibility while maintaining performance and reliability. Each layer has clear responsibilities and well-defined interfaces, making the system maintainable and extensible.
