//! Tests for JSON-LD conversion functionality

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rule::*, query::*, patch::*, strategy::*, catalog_jsonld::*};
    use serde_json::json;

    #[test]
    fn test_rule_ir_to_jsonld() {
        let rule = RuleIR::new("test_rule".to_string())
            .with_type("node1".to_string(), vec!["Label1".to_string()]);

        let jsonld = rule_ir_to_jsonld(&rule, Some("rule:test"));
        
        assert_eq!(jsonld["@type"], "kotoba:RuleIR");
        assert_eq!(jsonld["kotoba:name"], "test_rule");
        assert!(jsonld.get("kotoba:lhs").is_some());
        assert!(jsonld.get("kotoba:rhs").is_some());
    }

    #[test]
    fn test_rule_ir_from_jsonld() {
        let jsonld = json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@type": "kotoba:RuleIR",
            "kotoba:name": "test_rule",
            "kotoba:lhs": {
                "kotoba:nodes": [],
                "kotoba:edges": []
            },
            "kotoba:rhs": {
                "kotoba:nodes": [],
                "kotoba:edges": []
            }
        });

        let rule = rule_ir_from_jsonld(&jsonld).unwrap();
        assert_eq!(rule.name, "test_rule");
    }

    #[test]
    fn test_patch_ir_to_jsonld() {
        use crate::patch::*;
        let patch = Patch::empty()
            .add_vertex(AddVertex {
                id: "v1".to_string(),
                labels: vec!["Label1".to_string()],
                props: std::collections::HashMap::new(),
            });

        let jsonld = patch_ir_to_jsonld(&patch, Some("patch:test"));
        
        assert_eq!(jsonld["@type"], "kotoba:PatchIR");
        assert!(jsonld.get("kotoba:adds").is_some());
    }

    #[test]
    fn test_strategy_ir_to_jsonld() {
        let strategy = StrategyIR::new(StrategyOp::Once {
            rule: "rule1".to_string(),
        });

        let jsonld = strategy_ir_to_jsonld(&strategy, Some("strategy:test"));
        
        assert_eq!(jsonld["@type"], "kotoba:StrategyIR");
        assert!(jsonld.get("kotoba:strategy").is_some());
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

