//! DHT上でのEvolution Engine実装
//!
//! DHT上でEvolution Engineを実行します。

use crate::dht::{get_jsonld_entry, store_jsonld_entry};
use crate::provenance::HolochainProvenance;
use crate::types::{EvolutionEntry, EvolutionType};
use crate::Result;
use hdk::prelude::*;
use serde_json::{json, Value};

/// Holochain Evolution Engine
pub struct HolochainEvolutionEngine {
    provenance: HolochainProvenance,
}

impl HolochainEvolutionEngine {
    /// 新しいEvolution Engineを作成
    pub fn new() -> Self {
        Self {
            provenance: HolochainProvenance::new(),
        }
    }

    /// Evolution Engineを実行
    pub async fn evolve(
        &self,
        provenance_id: &str,
        evolution_type: EvolutionType,
    ) -> Result<EvolutionResult> {
        use crate::dht::{get_jsonld_entry, query_dht, resolve_cid, store_jsonld_entry};
        use crate::types::{ProvenanceEntry, DhtQuery};
        use kotoba_owl_reasoner::{ReasoningEngine, ReasoningLevel};
        use serde_json::json;

        // Provenanceデータを取得
        let provenance_entry_value = resolve_cid(provenance_id).await?;
        let provenance_entry: ProvenanceEntry = serde_json::from_value(provenance_entry_value)?;
        
        let provenance_jsonld = provenance_entry.provenance.clone();

        // OWL推論による最適化パターン発見
        let mut evolution_data = json!({
            "@type": "kotoba:Evolution",
            "kotoba:evolutionType": format!("{:?}", evolution_type),
            "kotoba:provenanceId": provenance_id,
        });

        if matches!(
            evolution_type,
            EvolutionType::PatternBased | EvolutionType::Hybrid
        ) {
            // OWL推論エンジンを作成
            let mut engine = ReasoningEngine::new(ReasoningLevel::OwlDl)
                .map_err(|e| crate::HolochainKotobasosError::Evolution(
                    format!("Failed to create reasoning engine: {}", e)
                ))?;

            // Provenanceデータをロード
            engine
                .load_ontology_from_jsonld(provenance_jsonld.clone())
                .await
                .map_err(|e| crate::HolochainKotobasosError::Evolution(
                    format!("Failed to load provenance: {}", e)
                ))?;

            // 推論を実行
            let reasoning_result = engine
                .reason()
                .await
                .map_err(|e| crate::HolochainKotobasosError::Evolution(
                    format!("Reasoning failed: {}", e)
                ))?;

            // 推論結果を取得
            let inferred = engine
                .inferred_triples_as_jsonld()
                .await
                .map_err(|e| crate::HolochainKotobasosError::Evolution(
                    format!("Failed to get inferred triples: {}", e)
                ))?;

            // パターンを抽出
            if let Some(patterns) = self.extract_patterns(&inferred).await {
                evolution_data["kotoba:patterns"] = json!(patterns);
            }
        }

        // パフォーマンスベースの進化
        if matches!(
            evolution_type,
            EvolutionType::PerformanceBased | EvolutionType::Hybrid
        ) {
            // メトリクスを分析
            let metrics = self.analyze_performance_metrics(&provenance_entry).await?;
            evolution_data["kotoba:metrics"] = json!(metrics);
        }

        // 進化提案を作成
        let evolution_entry = EvolutionEntry {
            id: format!("evolution:{}", uuid::Uuid::new_v4()),
            evolution: evolution_data.clone(),
            provenance_id: provenance_id.to_string(),
            evolution_type,
            created_at: chrono::Utc::now().timestamp(),
        };

        let entry_value = serde_json::to_value(&evolution_entry)?;
        let _entry_hash = store_jsonld_entry("Evolution", &entry_value).await?;

        Ok(EvolutionResult {
            evolution_id: evolution_entry.id.clone(),
            evolution_data: evolution_entry.evolution,
        })
    }

    /// パターンを抽出（OWL推論結果から）
    async fn extract_patterns(&self, inferred_jsonld: &Value) -> Option<Vec<Value>> {
        use kotoba_owl_reasoner::execute_sparql;
        use serde_json::json;

        let mut patterns = Vec::new();

        // パターン1: 頻繁に共起するプロセス
        let cooccurrence_query = r#"
            PREFIX kotoba: <https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            SELECT ?process1 ?process2 (COUNT(*) as ?count)
            WHERE {
                ?event1 prov:wasGeneratedBy ?process1 .
                ?event2 prov:wasGeneratedBy ?process2 .
                ?event1 prov:endedAtTime ?time1 .
                ?event2 prov:endedAtTime ?time2 .
                FILTER(?process1 != ?process2)
                FILTER(ABS(?time1 - ?time2) < 3600)
            }
            GROUP BY ?process1 ?process2
            HAVING (COUNT(*) > 3)
            ORDER BY DESC(?count)
            LIMIT 10
        "#;

        if let Ok(cooccurrence_result) = execute_sparql(cooccurrence_query, inferred_jsonld).await {
            if let Some(graph) = cooccurrence_result.get("@graph").and_then(|g| g.as_array()) {
                for pattern in graph {
                    patterns.push(json!({
                        "@type": "kotoba:CooccurrencePattern",
                        "kotoba:process1": pattern.get("process1"),
                        "kotoba:process2": pattern.get("process2"),
                        "holochain:frequency": pattern.get("count"),
                    }));
                }
            }
        }

        // パターン2: アクターパフォーマンスパターン
        let actor_perf_query = r#"
            PREFIX kotoba: <https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            SELECT ?actor ?process (AVG(?time) as ?avgTime) (COUNT(*) as ?count)
            WHERE {
                ?event prov:wasGeneratedBy ?process .
                ?event prov:wasAssociatedWith ?actor .
                ?event kotoba:executionTime ?time .
            }
            GROUP BY ?actor ?process
            HAVING (COUNT(*) > 2)
            ORDER BY ?avgTime
        "#;

        if let Ok(perf_result) = execute_sparql(actor_perf_query, inferred_jsonld).await {
            if let Some(graph) = perf_result.get("@graph").and_then(|g| g.as_array()) {
                for pattern in graph {
                    patterns.push(json!({
                        "@type": "kotoba:PerformancePattern",
                        "kotoba:actor": pattern.get("actor"),
                        "kotoba:process": pattern.get("process"),
                        "kotoba:avgExecutionTime": pattern.get("avgTime"),
                        "kotoba:executionCount": pattern.get("count"),
                    }));
                }
            }
        }

        if patterns.is_empty() {
            None
        } else {
            Some(patterns)
        }
    }

    /// パフォーマンスメトリクスを分析
    async fn analyze_performance_metrics(
        &self,
        provenance_entry: &ProvenanceEntry,
    ) -> Result<Value> {
        use serde_json::json;

        // Provenanceデータからメトリクスを抽出
        let metrics = json!({
            "process_id": provenance_entry.process_id,
            "executed_at": provenance_entry.executed_at,
            "executor": format!("{}", provenance_entry.executor),
        });

        Ok(metrics)
    }

    /// 進化履歴を取得
    pub async fn get_history(&self, story_id: &str) -> Result<Value> {
        use crate::dht::query_dht;
        use crate::types::{DhtQuery, EvolutionEntry};
        use serde_json::json;

        // story_idに関連するEvolutionエントリを検索
        let query = DhtQuery {
            entry_type: "Evolution".to_string(),
            filters: json!({
                "provenance_id": story_id  // 実際にはprovenance_idからstory_idを逆引きする必要がある
            }),
            pagination: None,
        };

        let results = query_dht(&query).await?;
        let mut evolution_history = Vec::new();

        for (_, entry_value) in results {
            if let Ok(evolution_entry) = serde_json::from_value::<EvolutionEntry>(entry_value) {
                evolution_history.push(evolution_entry.evolution);
            }
        }

        Ok(json!({
            "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
            "@type": "kotoba:EvolutionHistory",
            "@graph": evolution_history
        }))
    }
}

/// 進化結果
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub evolution_id: String,
    pub evolution_data: Value,
}

