//! # Kotoba Intermediate Representation (IR)
//!
//! Pure intermediate representation system for Kotoba, providing:
//! - catalog-IR: schema/index/invariant definitions
//! - rule-IR: DPO typed attribute graph rewriting
//! - query-IR: GQL logical plan algebra
//! - patch-IR: differential expressions
//! - strategy-IR: minimal strategy expressions
//!
//! All IR types are represented in JSON-LD format as the universal intermediate representation,
//! with OWL ontology definitions and SHACL shape validation support.

pub mod catalog_jsonld;
pub mod rule_jsonld;
pub mod query_jsonld;
pub mod patch_jsonld;
pub mod strategy_jsonld;
// Legacy Rust type modules (to be removed)
pub mod rule;
pub mod query;
pub mod patch;
pub mod strategy;
pub mod jsonld;

#[cfg(test)]
#[path = "jsonld_tests.rs"]
mod jsonld_tests;

// Re-export JSON-LD direct manipulation APIs (primary API)
pub use catalog_jsonld::*;
pub use rule_jsonld::*;
pub use query_jsonld::*;
pub use patch_jsonld::*;
pub use strategy_jsonld::*;
// Legacy re-exports (to be removed)
pub use rule::*;
pub use query::*;
pub use patch::*;
pub use strategy::*;
pub use jsonld::*;
