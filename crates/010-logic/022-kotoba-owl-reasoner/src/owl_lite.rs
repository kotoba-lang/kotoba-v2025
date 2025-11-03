//! OWL Lite reasoning implementation
//!
//! Provides OWL Lite reasoning using fukurow-lite tableau algorithm.

use crate::fukurow_binding::FukurowStore;
use crate::Result;
use fukurow_lite::OwlLiteReasoner;

/// Perform OWL Lite reasoning
/// 
/// Implements:
/// - Tableau algorithm (soundness and termination guaranteed)
/// - Class hierarchy inference (subsumption reasoning)
/// - Ontology consistency checking
pub async fn reason_owl_lite(store: &FukurowStore) -> Result<Vec<(String, String, String)>> {
    // Get store guard
    let rdf_store_guard = store.store_guard().await;
    
    // Create OWL Lite reasoner
    let mut reasoner = OwlLiteReasoner::new();
    
    // Load ontology from store
    let ontology = reasoner
        .load_ontology(&*rdf_store_guard)
        .map_err(|e| crate::OwlReasonerError::ReasoningError(e.to_string()))?;
    
    // Check consistency
    let consistent = reasoner
        .is_consistent(&ontology)
        .map_err(|e| crate::OwlReasonerError::ReasoningError(e.to_string()))?;
    
    if !consistent {
        return Err(crate::OwlReasonerError::ReasoningError(
            "Ontology is inconsistent".to_string(),
        ));
    }
    
    // Compute class hierarchy
    let hierarchy = reasoner
        .compute_class_hierarchy(&ontology)
        .map_err(|e| crate::OwlReasonerError::ReasoningError(e.to_string()))?;
    
    // Convert hierarchy to triples
    let mut triples = Vec::new();
    for (subclass, superclasses) in hierarchy {
        for superclass in superclasses {
            // Only add if subclass != superclass
            if subclass.iri.as_str() != superclass.iri.as_str() {
                triples.push((
                    subclass.iri.as_str().to_string(),
                    "http://www.w3.org/2000/01/rdf-schema#subClassOf".to_string(),
                    superclass.iri.as_str().to_string(),
                ));
            }
        }
    }
    
    Ok(triples)
}
