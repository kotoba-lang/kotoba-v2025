//! 単体テスト
//!
//! 各モジュールの単体テストを実装します。

use kotobas_tamaki_holochain::*;
use serde_json::json;

mod dht_tests {
    use super::*;

    #[tokio::test]
    async fn test_jsonld_to_cid() {
        let data = json!({
            "@type": "kotoba:Process",
            "@id": "kotoba:process/test",
            "kotoba:label": "Test Process"
        });

        let cid = utils::jsonld_to_cid(&data).unwrap();
        assert!(cid.starts_with("cid:"));
        assert_eq!(cid.len(), 66);
    }

    #[tokio::test]
    async fn test_cid_consistency() {
        let data = json!({
            "@type": "kotoba:Story",
            "@graph": []
        });

        let cid1 = utils::jsonld_to_cid(&data).unwrap();
        let cid2 = utils::jsonld_to_cid(&data).unwrap();
        
        assert_eq!(cid1, cid2);
    }
}

mod types_tests {
    use super::*;
    use types::*;

    #[test]
    fn test_story_entry_serialization() {
        let story_entry = StoryEntry {
            id: "cid:test".to_string(),
            story: json!({
                "@context": "https://example.com/context",
                "@graph": []
            }),
            created_at: 1234567890,
            author: hdk::prelude::AgentPubKey::from_raw_39(&[0u8; 39]).unwrap(),
        };

        let serialized = serde_json::to_value(&story_entry).unwrap();
        let deserialized: StoryEntry = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(story_entry.id, deserialized.id);
        assert_eq!(story_entry.created_at, deserialized.created_at);
    }

    #[test]
    fn test_process_entry_serialization() {
        let process_entry = ProcessEntry {
            id: "cid:process1".to_string(),
            process: json!({
                "@type": "kotoba:Process",
                "@id": "kotoba:process/test"
            }),
            story_id: "cid:story1".to_string(),
            created_at: 1234567890,
        };

        let serialized = serde_json::to_value(&process_entry).unwrap();
        let deserialized: ProcessEntry = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(process_entry.id, deserialized.id);
        assert_eq!(process_entry.story_id, deserialized.story_id);
    }

    #[test]
    fn test_evolution_type_serialization() {
        let evolution_type = EvolutionType::Hybrid;
        let serialized = serde_json::to_value(&evolution_type).unwrap();
        let deserialized: EvolutionType = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(evolution_type, deserialized);
    }
}

mod merkle_tests {
    use super::*;
    use merkle::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_merkle_node_creation() {
        let mut node = MerkleNode::new(
            "cid:test".to_string(),
            "hash:test".to_string(),
        );

        assert_eq!(node.id, "cid:test");
        assert_eq!(node.data_hash, "hash:test");
        assert!(node.parent_links.is_empty());
        assert!(node.child_links.is_empty());
    }

    #[tokio::test]
    async fn test_merkle_node_metadata() {
        let mut node = MerkleNode::new(
            "cid:test".to_string(),
            "hash:test".to_string(),
        );

        node.metadata = json!({
            "created_at": 1234567890,
            "is_root": true
        });

        assert_eq!(node.metadata["created_at"], 1234567890);
        assert_eq!(node.metadata["is_root"], true);
    }
}

mod mediator_tests {
    use super::*;
    use mediator::*;

    #[test]
    fn test_mediator_creation() {
        let mediator = HolochainMediator::new();
        assert_eq!(mediator.local_actors.len(), 0);
    }

    #[test]
    fn test_mediator_add_local_actor() {
        use types::ActorEntry;
        use hdk::prelude::AgentPubKey;

        let mut mediator = HolochainMediator::new();
        let actor = ActorEntry {
            id: "actor:1".to_string(),
            capability: "kotoba:capability/execution".to_string(),
            agent: AgentPubKey::from_raw_39(&[0u8; 39]).unwrap(),
            metadata: json!({}),
            registered_at: 1234567890,
        };

        mediator.add_local_actor(actor.clone());
        assert_eq!(mediator.local_actors.len(), 1);
        assert_eq!(mediator.local_actors[0].id, "actor:1");
    }
}

mod provenance_tests {
    use super::*;
    use provenance::*;

    #[test]
    fn test_provenance_creation() {
        let provenance = HolochainProvenance::new();
        // 基本的な作成テスト
        assert!(true);
    }
}

mod evolution_tests {
    use super::*;
    use evolution::*;

    #[test]
    fn test_evolution_engine_creation() {
        let engine = HolochainEvolutionEngine::new();
        // 基本的な作成テスト
        assert!(true);
    }

    #[test]
    fn test_evolution_result_serialization() {
        let result = EvolutionResult {
            evolution_id: "evolution:1".to_string(),
            evolution_data: json!({
                "@type": "kotoba:Evolution",
                "kotoba:evolutionType": "Hybrid"
            }),
        };

        let serialized = serde_json::to_value(&result).unwrap();
        assert_eq!(serialized["evolution_id"], "evolution:1");
    }
}

