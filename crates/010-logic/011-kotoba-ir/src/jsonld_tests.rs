//! Tests for JSON-LD direct manipulation API

#[cfg(test)]
mod tests {
    use crate::{rule_jsonld::*, query_jsonld::*, patch_jsonld::*, strategy_jsonld::*, catalog_jsonld::*};
    use serde_json::json;

    #[test]
    fn test_rule_jsonld_api() {
        let mut rule = create_empty_rule_jsonld(Some("rule:test"), "test_rule");
        
        // Set rule name
        set_rule_name(&mut rule, "updated_rule").unwrap();
        assert_eq!(get_rule_name(&rule), Some("updated_rule".to_string()));
        
        // Add type definition
        add_type_def(&mut rule, "node1", vec!["Label1"]).unwrap();
        let types = get_type_defs(&rule);
        assert!(types.is_some());
        
        // Add node to LHS pattern
        let mut lhs = get_lhs(&rule).unwrap();
        add_node_to_pattern(&mut lhs, "u", Some("V"), None).unwrap();
        set_lhs(&mut rule, lhs).unwrap();
        
        // Add edge to LHS pattern
        let mut lhs = get_lhs(&rule).unwrap();
        add_edge_to_pattern(&mut lhs, "e1", "u", "v", Some("E")).unwrap();
        set_lhs(&mut rule, lhs).unwrap();
        
        // Add guard
        add_guard(&mut rule, "deg_ge", json!({"var": "u", "k": 2})).unwrap();
        let guards = get_guards(&rule);
        assert!(guards.is_some());
        
        // Add NAC
        let nac = create_empty_nac();
        add_nac(&mut rule, nac).unwrap();
        let nacs = get_nacs(&rule);
        assert!(nacs.is_some());
        
        assert_eq!(rule["@type"], "kotoba:RuleIR");
    }

    #[test]
    fn test_query_jsonld_api() {
        let mut query = create_empty_query_jsonld(Some("query:test"));
        
        // Create a NodeScan operator
        let node_scan = create_node_scan("Person", "n", None);
        set_plan(&mut query, node_scan).unwrap();
        
        let plan = get_plan(&query);
        assert!(plan.is_some());
        assert_eq!(get_operator_type(&plan.unwrap()), Some("NodeScan".to_string()));
        
        // Create a Filter operator
        let filter = create_filter(
            json!({"ge": [{"fn": "degree", "args": ["n"]}, 50]}),
            plan.unwrap()
        );
        set_plan(&mut query, filter).unwrap();
        
        let plan = get_plan(&query);
        assert_eq!(get_operator_type(&plan.unwrap()), Some("Filter".to_string()));
    }

    #[test]
    fn test_patch_jsonld_api() {
        let mut patch = create_empty_patch_jsonld(Some("patch:test"));
        
        // Add a vertex
        add_vertex(&mut patch, "v1", vec!["Label1"], None).unwrap();
        
        // Add an edge
        add_edge(&mut patch, "e1", "v1", "v2", "RELATED_TO", None).unwrap();
        
        // Delete a vertex
        delete_vertex(&mut patch, "v3").unwrap();
        
        // Update a property
        update_property(&mut patch, "v1", "name", json!("Alice")).unwrap();
        
        // Relink an edge
        relink_edge(&mut patch, "e1", Some("v2"), Some("v3")).unwrap();
        
        assert_eq!(patch["@type"], "kotoba:PatchIR");
        assert!(!is_empty(&patch));
        
        // Verify adds
        let adds = get_adds(&patch);
        assert!(adds.is_some());
        
        // Verify dels
        let dels = get_dels(&patch);
        assert!(dels.is_some());
        
        // Verify updates
        let updates = get_updates(&patch);
        assert!(updates.is_some());
    }

    #[test]
    fn test_strategy_jsonld_api() {
        let mut strategy = create_empty_strategy_jsonld(Some("strategy:test"));
        
        // Create a Once strategy
        let once_op = create_once("rule1");
        set_strategy(&mut strategy, once_op).unwrap();
        
        let strategy_op = get_strategy(&strategy);
        assert!(strategy_op.is_some());
        assert_eq!(get_operator_type(&strategy_op.unwrap()), Some("Once".to_string()));
        
        // Create an Exhaust strategy
        let exhaust_op = create_exhaust("rule2", "topdown", Some("edge_count_nonincreasing"));
        set_strategy(&mut strategy, exhaust_op).unwrap();
        
        let strategy_op = get_strategy(&strategy);
        assert_eq!(get_operator_type(&strategy_op.unwrap()), Some("Exhaust".to_string()));
        
        // Create a Seq strategy
        let seq_op = create_seq(vec![create_once("rule1"), create_once("rule2")]);
        set_strategy(&mut strategy, seq_op).unwrap();
        
        let strategy_op = get_strategy(&strategy);
        assert_eq!(get_operator_type(&strategy_op.unwrap()), Some("Seq".to_string()));
    }

    #[test]
    fn test_catalog_jsonld_api() {
        let mut catalog = create_empty_catalog_jsonld(Some("catalog:test"));
        
        // Add a label definition
        let mut label_def = create_label_def_jsonld("Person", Some("label:Person"));
        let prop_def = create_property_def_jsonld("name", "string", false, None, None);
        add_property_def(&mut label_def, prop_def).unwrap();
        add_label_def(&mut catalog, label_def).unwrap();
        
        // Verify label was added
        let retrieved_label = get_label_def(&catalog, "Person");
        assert!(retrieved_label.is_some());
        
        // Add an index
        let index_def = create_index_def_jsonld("person_name_idx", "Person", vec!["name".to_string()], false, None);
        add_index_def(&mut catalog, index_def).unwrap();
        
        // Verify index was added
        let retrieved_index = get_index_def(&catalog, "person_name_idx");
        assert!(retrieved_index.is_some());
        
        // Add an invariant
        let invariant = create_invariant_jsonld("no_empty_name", "name != ''", "Name cannot be empty", None);
        add_invariant(&mut catalog, invariant).unwrap();
        
        // Verify invariant was added
        let retrieved_invariant = get_invariant(&catalog, "no_empty_name");
        assert!(retrieved_invariant.is_some());
    }
}

