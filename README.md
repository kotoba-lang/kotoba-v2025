# Kotoba: A Phonosemantic Digital Computing System

## Legacy Reference

`kotoba-v2025` is intentionally preserved as a historical Rust design/reference
workspace. It is not part of the active Kotoba/CLJC migration path, and its
Cargo workspace is expected to remain here for archaeology and comparison.

Active language, CLI, db, git/rad, deploy, and contract authority should live in
current Kotoba/CLJC/EDN repositories such as `kotoba-lang/kotoba`,
`kotoba-lang/kotoba-lang`, and focused domain contract repos.

Ἐν ἀρχῇ ἦν ὁ λόγος

**Kotoba is a phonosemantic digital computing system where all computing, operating system, datastore, and self-evolution mechanisms are represented, reasoned, and executed using JSON-LD with OWL inference.**

This project integrates three foundational concepts:
1.  **Phonosemantic Vocabulary System**: A systematic mapping between phonemes (sound units) and semantic meanings, enabling natural language understanding through structured vocabulary systems.
2.  **OWL Inference Engine**: Complete reasoning capabilities using RDFS, OWL Lite, and OWL DL inference engines (powered by [fukurow](https://github.com/com-junkawasaki/fukurow)) for logical deduction and knowledge discovery.
3.  **Semantic Execution Pattern**: A Kernel + Actor + Mediator architecture (inspired by [semanticos](https://github.com/com-junkawasaki/semanticos)) for executing process networks defined in JSON-LD with automatic actor selection and provenance tracking.

Together, they form a cohesive ecosystem where computation is expressed semantically and reasoned automatically.

```bash
🔤 Phonosemantic Vocabulary Mapping
🦉 OWL Inference (RDFS + OWL Lite + OWL DL)
📦 JSON-LD Native (All Computing Layers)
🔄 Self-Evolution via Semantic Design Loop
🏗️ Kernel + Actor + Mediator Pattern
```

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/workflow/status/jun784/kotoba/CI)](https://github.com/com-junkawasaki/kotoba/actions)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Architecture](https://img.shields.io/badge/Architecture-Pure%20Kernel%20%26%20Effects%20Shell-blue)](#-architecture-pure-kernel--effects-shell)

## 📖 Vision: Phonosemantic Digital Computing

Kotoba reimagines computing through the lens of **phonosemantics** (音素意味論) and **semantic reasoning**. Every aspect of computation—from low-level operations to high-level application logic—is expressed, reasoned, and executed using JSON-LD with OWL inference.

### Core Principles

1. **Phonosemantic Vocabulary System**
   - Systematic mapping between phonemes (sound units) and semantic meanings
   - Natural language understanding through structured vocabulary relationships
   - Bidirectional conversion: phoneme → meaning and meaning → phoneme

2. **OWL Inference-Based Reasoning**
   - **Reasoner Layer (014)**: Uses [fukurow](https://github.com/com-junkawasaki/fukurow) as the core OWL reasoning engine
   - **RDFS Inference**: Transitive closure of subClassOf and subPropertyOf
   - **OWL Lite Inference**: Tableau algorithm for consistency checking and subsumption reasoning
   - **OWL DL Inference**: Extended tableau algorithm for complete reasoning
   - **SHACL Validation**: Shape constraint validation for RDF graphs
   - **SPARQL Queries**: SPARQL 1.1 query execution
   - Powered by fukurow, a WebAssembly-native OWL reasoning engine providing complete RDFS, OWL Lite, and OWL DL reasoning capabilities

3. **JSON-LD as Universal Representation**
   - All computing layers use JSON-LD: hardware operations, OS services, datastore operations, and self-evolution mechanisms
   - **JSON-LD Universal IR**: All Intermediate Representations (Rule-IR, Query-IR, Patch-IR, Strategy-IR, Catalog-IR) are represented in JSON-LD format
   - **OWL Ontology Definitions**: IR type hierarchy defined using OWL classes (`kotoba:IR`, `kotoba:RuleIR`, etc.)
   - **SHACL Shape Validation**: All IR operations automatically validate against SHACL shapes (enabled via `reasoning` feature)
   - **WASM Runtime Integration**: JSON-LD IRs can be executed in WebAssembly runtime for high-performance execution (enabled via `wasm` feature)
   - Unified semantic representation eliminates syntactic barriers between layers
   - Automatic context resolution and vocabulary expansion

4. **Semantic Execution Pattern (semanticos)**
   - **Kernel**: Orchestrates process network execution
   - **Actor**: Performs actions based on capabilities
   - **Mediator**: Selects appropriate actors using OWL reasoning-based capability matching
   - **Capability System**: OWL-based capability definitions with SHACL validation for actor-process matching
   - **Provenance**: Records all execution history in JSON-LD format

5. **Self-Evolution via Semantic Design Loop**
   - System continuously improves itself by analyzing provenance
   - OWL inference discovers optimization patterns
   - Automatic shape refinement and process optimization

6. **Storage Adapter Architecture**
   - **Storage Layer (030)**: Implements Port/Adapter pattern for storage abstraction
   - **Primary Adapter**: [fcdb](https://github.com/com-junkawasaki/fcdb) (Functorial-Categorical Database) for content-addressable storage with graph capabilities
   - **MVCC + Merkle DAG**: Consistent distributed data management with CID-based addressing
   - **Multiple Adapters**: Redis, RocksDB, Memory, GraphDB implementations available
   - All storage adapters implement the `StorageEngine` trait for seamless switching

This approach enables **semantic interoperability** across all computing layers, where meaning is preserved and reasoned about automatically.

## 🏗️ Architecture: Phonosemantic Computing Stack

The entire system is built upon a **Semantic-Driven Architecture**, where all layers communicate via JSON-LD and reason using OWL inference.

### Layer Architecture

Kotoba follows a layered architecture with clear separation of concerns:

```
Layer 090: Tools Layer          (Development tools, CLI, build tools)
Layer 080: Deployment Layer     (Deployment, scaling, networking)
Layer 070: Services Layer       (HTTP/GraphQL servers, external integrations)
Layer 060: Application Layer    (Business logic, event sourcing, query processing)
Layer 050: Workflow Layer       (Workflow orchestration)
Layer 040: Runtime Layer        (OS + Storage + Reasoner integration)
Layer 030: Storage Layer        (Persistence, MVCC+Merkle DAG: fcdb adapter)
Layer 020: Language Layer       (Parser, Analyzer, Transpiler)
Layer 015: OS Layer            (Process network orchestration)
Layer 014: Reasoner Layer      (OWL reasoning engine: fukurow)
Layer 012: VM Layer            (Virtual Machine execution environment)
Layer 010: Logic Layer         (IR, Rewrite Kernel, JSON-LD)
Layer 005: Foundation Layer    (Types, CID, Schema, Auth, Graph Core)
```

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
│                    OWL Inference Layer                       │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │ RDFS Reasoner│   │ OWL Lite     │   │ OWL DL      │      │
│ │ (Transitive) │   │ (Tableau)    │   │ (Extended)  │      │
│ └──────┬───────┘   └──────┬───────┘   └──────┬───────┘      │
│        └──────────────────┼──────────────────┘            │
│                            │ fukurow                          │
└────────────────────────────┼────────────────────────────────┘
                             │ JSON-LD Triples
┌────────────────────────────┼────────────────────────────────┐
│                   JSON-LD Data Layer                         │
│ ┌──────────────┐   ┌──────────────┐   ┌──────────────┐      │
│ │ Phonosemantic│   │  Vocabulary │   │  Process    │      │
│ │   Mapping    │   │   System    │   │  Network    │      │
│ └──────────────┘   └──────────────┘   └──────────────┘      │
│                                                              │
│              All data structures are JSON-LD                │
│         with @context, @id, @type, and kotoba: prefixes      │
└─────────────────────────────────────────────────────────────┘
```

## 📁 Unified Project Structure

The project is a modular multi-crate workspace, separating the low-level computing system from the high-level application framework.

```
├── crates/
│   ├── 005-foundation/           # Foundation Layer: Types, CID, Schema, Auth, Graph Core
│   ├── 010-logic/                # Logic Layer: IR, Rewrite Kernel, JSON-LD
│   │   ├── 011-kotoba-ir/        # JSON-LD Universal IR (Rule, Query, Patch, Strategy, Catalog)
│   │   ├── 012-kotoba-rewrite-kernel/  # DPO graph rewriting kernel
│   │   ├── 019-kotoba-jsonld/   # JSON-LD processing utilities
│   │   └── 022-kotoba-owl-reasoner/  # OWL reasoning engine (fukurow integration)
│   ├── 012-vm/                   # VM Layer: Virtual Machine execution environment
│   ├── 014-reasoner/             # Reasoner Layer: OWL reasoning (fukurow)
│   ├── 015-os/                   # OS Layer: Process network orchestration
│   ├── 020-language/             # Language Layer: Parser, Analyzer, Transpiler
│   ├── 030-storage/              # Storage Layer: Persistence adapters
│   │   ├── 031-kotoba-storage/   # Storage interface (Port/Adapter pattern)
│   │   ├── 039-kotoba-storage-fcdb/  # FCDB adapter (primary: content-addressable graph storage)
│   │   ├── 037-kotoba-storage-redis/  # Redis adapter
│   │   └── 038-kotoba-storage-rocksdb/  # RocksDB adapter
│   ├── 040-runtime/              # Runtime Layer: OS + Storage + Reasoner integration
│   ├── 050-workflow/             # Workflow Layer: Workflow orchestration
│   ├── 060-application/          # Application Layer: Business logic, event sourcing
│   ├── 070-services/             # Services Layer: HTTP/GraphQL servers
│   ├── 080-deployment/           # Deployment Layer: Deployment, scaling, networking
│   └── 090-tools/                # Tools Layer: Development tools, CLI, build tools
├── kotoba-cli/                   # Main CLI for the Kotoba ecosystem
└── kotoba-server/                # Effects Shell implementation for the HTTP server
```

## 🎯 Key Components

### 1. The Kotoba VM - Tamaki Architecture (`005-vm`)

A high-performance virtual machine that forms the execution layer of Kotoba.

-   **GNN Optimization Engine**: Uses a Program Interaction Hypergraph (PIH) to apply learned, hardware-specific optimizations via safe DPO graph rewriting.
-   **Content-Addressable**: Employs a CID (Content ID) system with Blake3 hashing to create a verifiable Merkle-DAG of all computations.
-   **Heterogeneous Execution**: Simulates and schedules tasks across diverse hardware tiles like CPUs, GPUs, and specialized accelerators (CGRA/FPGA).
-   **High Performance**: Backed by extensive benchmarks demonstrating significant speedups over traditional approaches.

**🚀 Validated Performance of the Kotoba VM - Tamaki Architecture:**
- **DAG Scheduling**: **5.7x** faster than simple topological sort.
- **Memory Efficiency**: **35x** better sequential access performance.
- **Memoization**: **78-85%** cache hit rates.
- **Network Efficiency**: **288x** improvement over pure ring topology at 65k nodes.
- **Energy Savings**: **35-45%** reduction compared to traditional systems.
- **Case Studies**: **2.3x-4.7x** performance improvements across ETL, ML, video analytics, and scientific simulation.

### 2. Phonosemantic Vocabulary System

The systematic mapping system that connects phonemes to semantic meanings.

-   **Phoneme Structure**: Represents sound units with phonetic features
-   **Semantic Mapping**: Bidirectional conversion between phonemes and meanings
-   **Vocabulary Management**: JSON-LD-based vocabulary system with OWL inference
-   **Natural Language Integration**: Enables natural language understanding through structured relationships

### 3. OWL Inference Engine (fukurow)

Complete OWL reasoning capabilities integrated into Kotoba, powered by [fukurow](https://github.com/com-junkawasaki/fukurow).

-   **Reasoner Layer (014)**: Core reasoning engine using fukurow
-   **RDFS Inference**: Transitive closure computation for class and property hierarchies
-   **OWL Lite Reasoning**: Tableau algorithm for consistency checking and subsumption
-   **OWL DL Reasoning**: Extended tableau algorithm for complete logical reasoning
-   **SHACL Validation**: Shape constraint validation for RDF graphs
-   **SPARQL Queries**: SPARQL 1.1 query execution
-   **JSON-LD Integration**: Native support for JSON-LD input/output
-   **WebAssembly Support**: Browser-ready inference engine

### 4. Storage Adapter Architecture (fcdb)

Port/Adapter pattern implementation for storage abstraction, with [fcdb](https://github.com/com-junkawasaki/fcdb) as the primary adapter.

-   **Storage Layer (030)**: Implements `StorageEngine` trait for pluggable storage backends
-   **Primary Adapter**: FCDB (Functorial-Categorical Database) for content-addressable storage with graph capabilities
-   **MVCC + Merkle DAG**: Consistent distributed data management with CID-based addressing
-   **Graph Database**: Built-in graph operations with RID (Resource ID) and LabelId support
-   **Additional Adapters**: Redis, RocksDB, Memory, GraphDB implementations available
-   **Seamless Switching**: All adapters implement the same interface for runtime selection

### 5. Semantic Execution Pattern (semanticos)

Process network execution with automatic actor selection and provenance tracking.

-   **Kernel**: Orchestrates process network graph execution
-   **Mediator**: Selects actors using OWL reasoning-based capability matching
-   **Actor**: Performs actions based on process requirements and capabilities
-   **Provenance**: Records all execution history in JSON-LD/PROV-O format

## 🚀 Quick Start

### Prerequisites

-   **Rust 1.70.0 or later**
-   **Cargo package manager**

### Installation & Usage

```bash
# Clone the repository
git clone https://github.com/com-junkawasaki/kotoba.git
cd kotoba

# Build the entire project workspace
cargo build --release --workspace

# Run the comprehensive test suite for all crates
cargo test --workspace

# Run the main CLI
./target/release/kotoba-cli --help

# Run the VM-specific benchmarks
cargo bench --package vm-benchmarks
```

## 🤝 Contributing

This project aims to redefine computing from the ground up. Contributions are welcome, from low-level VM optimizations to high-level language features.

1.  **Fork the repository**
2.  **Create a feature branch** (`git checkout -b feature/your-feature`)
3.  **Commit your changes** (`git commit -m 'feat: Add some feature'`)
4.  **Push to the branch** (`git push origin feature/your-feature`)
5.  **Open a Pull Request**

## 📄 License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
