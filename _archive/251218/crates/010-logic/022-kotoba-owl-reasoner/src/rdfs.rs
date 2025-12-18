//! RDFS reasoning implementation
//!
//! Provides RDFS-level reasoning using fukurow-rdfs.

use crate::fukurow_binding::FukurowStore;
use crate::Result;
use fukurow_rdfs::RdfsReasoner;
use fukurow_store::store::RdfStore;

/// Perform RDFS reasoning
/// 
/// Implements:
/// - rdfs:subClassOf transitive closure
/// - rdfs:subPropertyOf transitive closure
/// - rdfs:domain and rdfs:range type inference
/// - rdf:type inference and hierarchical type propagation
pub async fn reason_rdfs(store: &FukurowStore) -> Result<Vec<(String, String, String)>> {
    let rdf_store = store.store().lock().await;
    
    // Create RDFS reasoner
    let mut reasoner = RdfsReasoner::new();
    
    // Compute closure
    let inferred_triples = reasoner
        .compute_closure(&*rdf_store)
        .map_err(|e| crate::OwlReasonerError::ReasoningError(e.to_string()))?;
    
    // Convert to (subject, predicate, object) tuples
    Ok(inferred_triples
        .into_iter()
        .map(|t| (t.subject, t.predicate, t.object))
        .collect())
}
