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
pub mod rule;
pub mod query;
pub mod patch;
pub mod strategy;
pub mod jsonld;

#[cfg(test)]
#[path = "jsonld_tests.rs"]
mod jsonld_tests;

// Re-export everything for convenience
pub use catalog_jsonld::*;
pub use rule::*;
pub use query::*;
pub use patch::*;
pub use strategy::*;
pub use jsonld::*;

// Core IR types
pub use crate::rule::*;
pub use crate::query::*;
pub use crate::patch::*;
pub use crate::strategy::*;
pub use crate::jsonld::*;
