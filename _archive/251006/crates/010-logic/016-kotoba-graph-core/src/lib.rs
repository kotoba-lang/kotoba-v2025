//! # Kotoba Graph Core
//!
//! Incidence bipartite graph + canonicalization + merkle for content-addressed graphs.
//!
//! This crate provides the foundation for content-addressed graphs using
//! incidence bipartite representation with canonicalization and merkle hashing.

pub mod incidence;
pub mod canonical;
pub mod merkle;
pub mod graph;
pub mod algorithms;

use kotoba_types::{Hash as KotobaHash, *};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Use KotobaHash to avoid conflict with std::hash::Hash trait
pub type Hash = KotobaHash;

pub use graph::Graph;
pub use algorithms::GraphStatistics;
pub use canonical::GraphCanonicalizer;
// CanonicalizationAlgorithm, CanonicalizationConfig, CanonicalizationResult are defined in this file (lib.rs)
// MerkleTree is defined in this file (lib.rs)
pub use merkle::{MerkleTreeBuilder, MerkleConfig};

/// Graph ID type for content addressing
pub type GraphId = Hash;

/// Content-addressed graph reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphRef {
    /// Content hash of the graph
    pub hash: Hash,
    /// Canonical form of the graph
    pub canonical_form: Vec<u8>,
}

impl GraphRef {
    /// Create a new graph reference
    pub fn new(hash: Hash, canonical_form: Vec<u8>) -> Self {
        Self { hash, canonical_form }
    }

    /// Create from graph content
    pub fn from_graph<T: AsRef<[u8]>>(content: T) -> Self {
        let hash = Hash::from_sha256(content.as_ref());
        let canonical_form = content.as_ref().to_vec();
        Self::new(hash, canonical_form)
    }
}

/// Graph canonicalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizationResult {
    /// Canonical graph representation
    pub canonical_graph: Vec<u8>,
    /// Hash of the canonical form
    pub hash: Hash,
    /// Isomorphism class ID
    pub isomorphism_class: String,
    /// Canonical ordering of nodes
    pub node_ordering: Vec<usize>,
    /// Canonical ordering of edges
    pub edge_ordering: Vec<usize>,
}

/// Merkle tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Hash of this node
    pub hash: Hash,
    /// Left child
    pub left: Option<Box<MerkleNode>>,
    /// Right child
    pub right: Option<Box<MerkleNode>>,
    /// Data at this node (for leaf nodes)
    pub data: Option<Vec<u8>>,
}

/// Merkle tree for graph integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// Root node
    pub root: Option<MerkleNode>,
    /// Height of the tree
    pub height: usize,
    /// Number of leaves
    pub leaf_count: usize,
}

// MerkleTree implementation is in merkle.rs

// GraphCanonicalizer is defined in canonical.rs

/// Canonicalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizationConfig {
    /// Algorithm to use for canonicalization
    pub algorithm: CanonicalizationAlgorithm,
    /// Maximum graph size for canonicalization
    pub max_size: Option<usize>,
    /// Enable optimizations
    pub enable_optimizations: bool,
}

impl Default for CanonicalizationConfig {
    fn default() -> Self {
        Self {
            algorithm: CanonicalizationAlgorithm::Bliss,
            max_size: Some(10000),
            enable_optimizations: true,
        }
    }
}

/// Canonicalization algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanonicalizationAlgorithm {
    /// Bliss canonical labeling
    Bliss,
    /// Nauty canonical labeling
    Nauty,
    /// Custom algorithm
    Custom(String),
}

/// Incidence bipartite graph representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncidenceGraph {
    /// Left vertices (entities)
    pub left_vertices: Vec<IncidenceVertex>,
    /// Right vertices (attributes/relations)
    pub right_vertices: Vec<IncidenceVertex>,
    /// Edges between left and right
    pub edges: Vec<IncidenceEdge>,
}

impl IncidenceGraph {
    /// Create a new incidence graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a left vertex (returns new graph)
    pub fn with_left_vertex(self, vertex: IncidenceVertex) -> Self {
        let mut new_graph = self.clone();
        new_graph.left_vertices.push(vertex);
        new_graph
    }

    /// Add a right vertex (returns new graph)
    pub fn with_right_vertex(self, vertex: IncidenceVertex) -> Self {
        let mut new_graph = self.clone();
        new_graph.right_vertices.push(vertex);
        new_graph
    }

    /// Add an edge (returns new graph)
    pub fn with_edge(self, edge: IncidenceEdge) -> Self {
        let mut new_graph = self.clone();
        new_graph.edges.push(edge);
        new_graph
    }
}

/// Incidence vertex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidenceVertex {
    /// Vertex ID
    pub id: String,
    /// Vertex type
    pub vertex_type: String,
    /// Properties
    pub properties: HashMap<String, Value>,
}

/// Incidence edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidenceEdge {
    /// Source vertex (left)
    pub source: String,
    /// Target vertex (right)
    pub target: String,
    /// Edge type
    pub edge_type: String,
    /// Properties
    pub properties: HashMap<String, Value>,
}

/// Graph processor for processing graphs with canonicalization and merkle
#[derive(Debug, Clone)]
pub struct GraphProcessor {
    /// Canonicalizer
    pub canonicalizer: canonical::GraphCanonicalizer,
    /// Merkle tree builder
    pub merkle_builder: merkle::MerkleTreeBuilder,
}

impl GraphProcessor {
    /// Create a new graph processor
    pub fn new() -> Self {
        Self {
            canonicalizer: canonical::GraphCanonicalizer::new(CanonicalizationAlgorithm::Bliss),
            merkle_builder: merkle::MerkleTreeBuilder::new(MerkleConfig::default()),
        }
    }

    /// Process a graph: canonicalize and compute merkle tree
    pub fn process_graph(&self, graph: &Graph) -> GraphProcessingResult {
        // Canonicalize the graph
        let canonicalization = self.canonicalizer.canonicalize(graph);

        // Clone values before moving
        let hash = canonicalization.hash.clone();
        let canonical_graph = canonicalization.canonical_graph.clone();

        // Build merkle tree from canonical form
        // Split canonical graph into chunks for merkle tree
        let chunk_size = self.merkle_builder.config.chunk_size;
        let chunks: Vec<Vec<u8>> = canonicalization.canonical_graph.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        let merkle_tree = MerkleTree::new(chunks);

        GraphProcessingResult {
            canonicalization,
            merkle_tree,
            graph_ref: GraphRef::new(
                hash,
                canonical_graph,
            ),
        }
    }
}


/// Graph processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphProcessingResult {
    /// Canonicalization result
    pub canonicalization: CanonicalizationResult,
    /// Merkle tree
    pub merkle_tree: MerkleTree,
    /// Graph reference
    pub graph_ref: GraphRef,
}
