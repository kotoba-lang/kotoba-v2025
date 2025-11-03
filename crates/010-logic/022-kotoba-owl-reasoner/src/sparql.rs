//! SPARQL query implementation
//!
//! Provides SPARQL 1.1 query execution using fukurow-sparql.

use crate::fukurow_binding::FukurowStore;
use crate::Result;
use fukurow_shacl::loader::{DefaultShaclLoader, ShaclLoader};
use fukurow_shacl::loader::{Shape, NodeShape, PropertyShape, PropertyPath, PropertyConstraint, Target};
use fukurow_sparql::execute_query;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Execute a SPARQL query on JSON-LD data
pub async fn execute_sparql(query: &str, data: &Value) -> Result<Value> {
    // Convert JSON-LD data to RdfStore
    let mut store = FukurowStore::new();
    store.load_from_jsonld(data.clone()).await?;
    
    // Execute SPARQL query
    let rdf_store = store.store_guard().await;
    let query_result = execute_query(query, &rdf_store)
        .map_err(|e| crate::OwlReasonerError::Other(anyhow::anyhow!("SPARQL execution failed: {}", e)))?;
    
    drop(rdf_store);
    
    // Convert query result to JSON-LD
    match query_result {
        fukurow_sparql::QueryResult::Select { variables, bindings } => {
            let mut graph = Vec::new();
            for binding in bindings {
                let mut node = serde_json::Map::new();
                for var in &variables {
                    if let Some(term) = binding.get(var) {
                        match term {
                            fukurow_sparql::parser::Term::Iri(iri) => {
                                node.insert(var.0.clone(), json!(iri.0));
                            }
                            fukurow_sparql::parser::Term::Literal(lit) => {
                                node.insert(var.0.clone(), json!(lit.value));
                            }
                            fukurow_sparql::parser::Term::Variable(_) => {
                                // Variables shouldn't appear in bindings
                            }
                        }
                    }
                }
                if !node.is_empty() {
                    graph.push(Value::Object(node));
                }
            }
            
            Ok(json!({
                "@context": {
                    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                },
                "@graph": graph
            }))
        }
        fukurow_sparql::QueryResult::Construct { triples } => {
            let graph: Vec<Value> = triples.iter().map(|t| {
                json!({
                    "@id": t.subject,
                    "@type": if t.predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                        t.object.clone()
                    } else {
                        null
                    },
                    [t.predicate.clone()]: t.object.clone()
                })
            }).collect();
            
            Ok(json!({
                "@context": {
                    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                },
                "@graph": graph
            }))
        }
        fukurow_sparql::QueryResult::Ask { result } => {
            Ok(json!({
                "@context": {
                    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                },
                "result": result
            }))
        }
        fukurow_sparql::QueryResult::Describe { triples } => {
            let graph: Vec<Value> = triples.iter().map(|t| {
                json!({
                    "@id": t.subject,
                    [t.predicate.clone()]: t.object.clone()
                })
            }).collect();
            
            Ok(json!({
                "@context": {
                    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                },
                "@graph": graph
            }))
        }
    }
}

/// Compile SHACL shape to SPARQL query
pub async fn compile_shape_to_sparql(shape: &Value) -> Result<String> {
    // Convert JSON-LD shape to RdfStore and load ShapesGraph
    let mut shape_store = FukurowStore::new();
    shape_store.load_from_jsonld(shape.clone()).await?;
    
    let loader = DefaultShaclLoader;
    let rdf_store = shape_store.store_guard().await;
    let shapes_graph = loader.load_from_store(&rdf_store)
        .map_err(|e| crate::OwlReasonerError::Other(anyhow::anyhow!("Failed to load SHACL shapes: {}", e)))?;
    
    drop(rdf_store);
    
    // Compile shapes to SPARQL
    let mut query_parts = Vec::new();
    let mut prefixes = HashSet::new();
    prefixes.insert(("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"));
    prefixes.insert(("rdfs", "http://www.w3.org/2000/01/rdf-schema#"));
    prefixes.insert(("sh", "http://www.w3.org/ns/shacl#"));
    
    // Process each shape
    for (shape_id, shape) in &shapes_graph.shapes {
        match shape {
            Shape::Node(node_shape) => {
                let node_query = compile_node_shape_to_sparql(node_shape, &shapes_graph, &mut prefixes)?;
                query_parts.push(node_query);
            }
            Shape::Property(_prop_shape) => {
                // Property shapes are handled within NodeShape compilation
            }
        }
    }
    
    // Build final query
    let mut query = String::new();
    
    // Add prefixes
    for (prefix, namespace) in prefixes {
        query.push_str(&format!("PREFIX {}: <{}>\n", prefix, namespace));
    }
    
    query.push_str("SELECT ?s WHERE {\n");
    
    if query_parts.is_empty() {
        query.push_str("    ?s ?p ?o .\n");
    } else {
        // Combine all query parts with UNION
        for (i, part) in query_parts.iter().enumerate() {
            if i > 0 {
                query.push_str("    UNION\n");
            }
            query.push_str(&format!("    {{\n{}\n    }}\n", indent_query(part, 2)));
        }
    }
    
    query.push_str("}\n");
    
    Ok(query)
}

/// Compile a NodeShape to SPARQL query
fn compile_node_shape_to_sparql(
    node_shape: &NodeShape,
    shapes_graph: &fukurow_shacl::loader::ShapesGraph,
    prefixes: &mut HashSet<(&'static str, &'static str)>,
) -> Result<String> {
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    
    // Handle targets
    for target in &node_shape.targets {
        match target {
            Target::Class(class) => {
                patterns.push(format!("?s rdf:type <{}> .", class.0));
            }
            Target::Node(node) => {
                patterns.push(format!("?s = <{}> .", node.0));
            }
            Target::SubjectsOf(predicate) => {
                patterns.push(format!("?s <{}> ?o .", predicate.0));
            }
            Target::ObjectsOf(predicate) => {
                patterns.push(format!("?o <{}> ?s .", predicate.0));
            }
        }
    }
    
    // Handle property shapes
    for prop_shape_id in &node_shape.property_shapes {
        if let Some(prop_shape) = shapes_graph.get_shape(prop_shape_id) {
            if let Shape::Property(prop_shape) = prop_shape {
                let (prop_patterns, prop_filters) = compile_property_shape_to_sparql(prop_shape, prefixes)?;
                patterns.extend(prop_patterns);
                filters.extend(prop_filters);
            }
        }
    }
    
    // Combine patterns and filters
    let mut query = String::new();
    for pattern in patterns {
        query.push_str(&format!("  {}\n", pattern));
    }
    for filter in filters {
        query.push_str(&format!("  FILTER({})\n", filter));
    }
    
    Ok(query)
}

/// Compile a PropertyShape to SPARQL patterns and filters
fn compile_property_shape_to_sparql(
    prop_shape: &PropertyShape,
    prefixes: &mut HashSet<(&'static str, &'static str)>,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    
    // Compile property path
    let path_pattern = compile_property_path_to_sparql(&prop_shape.path, "?s", "?value")?;
    patterns.push(path_pattern);
    
    // Compile constraints
    for constraint in &prop_shape.constraints {
        match constraint {
            PropertyConstraint::MinCount(min_count) => {
                // Count occurrences and filter
                filters.push(format!("(COUNT(?value) >= {})", min_count));
            }
            PropertyConstraint::MaxCount(max_count) => {
                filters.push(format!("(COUNT(?value) <= {})", max_count));
            }
            PropertyConstraint::Datatype(datatype) => {
                // Check datatype
                patterns.push(format!("?value rdf:type <{}> .", datatype.0));
            }
            PropertyConstraint::Class(class) => {
                // Check if value is instance of class
                patterns.push(format!("?value rdf:type <{}> .", class.0));
            }
            PropertyConstraint::HasValue(expected_value) => {
                filters.push(format!("?value = \"{}\"", expected_value));
            }
            PropertyConstraint::MinLength(min_length) => {
                filters.push(format!("(STRLEN(?value) >= {})", min_length));
            }
            PropertyConstraint::MaxLength(max_length) => {
                filters.push(format!("(STRLEN(?value) <= {})", max_length));
            }
            PropertyConstraint::Pattern { pattern, flags: _ } => {
                // SPARQL REGEX function
                filters.push(format!("REGEX(?value, \"{}\")", pattern.replace("\\", "\\\\")));
            }
            _ => {
                // Other constraints not yet implemented
            }
        }
    }
    
    Ok((patterns, filters))
}

/// Compile a PropertyPath to SPARQL property path expression
fn compile_property_path_to_sparql(
    path: &PropertyPath,
    subject: &str,
    object: &str,
) -> Result<String> {
    match path {
        PropertyPath::Predicate(predicate) => {
            Ok(format!("{} <{}> {} .", subject, predicate.0, object))
        }
        PropertyPath::Inverse(inner) => {
            let inner_pattern = compile_property_path_to_sparql(inner, object, subject)?;
            Ok(inner_pattern.replace(subject, object).replace(object, subject))
        }
        PropertyPath::Sequence(paths) => {
            // Chain property paths
            let mut vars = Vec::new();
            for (i, _) in paths.iter().enumerate() {
                vars.push(format!("?var{}", i));
            }
            vars.push(object.to_string());
            
            let mut patterns = Vec::new();
            let mut current_subject = subject.to_string();
            for (i, path) in paths.iter().enumerate() {
                let next_var = &vars[i + 1];
                let pattern = compile_property_path_to_sparql(path, &current_subject, next_var)?;
                patterns.push(pattern);
                current_subject = next_var.clone();
            }
            
            Ok(patterns.join("\n  "))
        }
        PropertyPath::Alternative(paths) => {
            // UNION of paths
            let alternatives: Vec<String> = paths.iter()
                .map(|p| compile_property_path_to_sparql(p, subject, object))
                .collect::<Result<Vec<_>>>()?;
            
            Ok(format!("({})", alternatives.join(" | ")))
        }
        PropertyPath::ZeroOrMore(inner) => {
            let inner_pattern = compile_property_path_to_sparql(inner, subject, object)?;
            Ok(format!("{}*", inner_pattern))
        }
        PropertyPath::OneOrMore(inner) => {
            let inner_pattern = compile_property_path_to_sparql(inner, subject, object)?;
            Ok(format!("{}+", inner_pattern))
        }
        PropertyPath::ZeroOrOne(inner) => {
            let inner_pattern = compile_property_path_to_sparql(inner, subject, object)?;
            Ok(format!("{}?", inner_pattern))
        }
    }
}

/// Indent a query string
fn indent_query(query: &str, indent_level: usize) -> String {
    let indent = " ".repeat(indent_level);
    query.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

