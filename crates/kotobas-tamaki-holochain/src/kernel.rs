//! Holochain対応Kernel実装
//!
//! 既存の`kotoba_os::Kernel`を拡張し、Holochainのエージェント中心モデルに適応します。

use crate::dht::{get_jsonld_entry, store_jsonld_entry};
use crate::evolution::HolochainEvolutionEngine;
use crate::mediator::HolochainMediator;
use crate::provenance::HolochainProvenance;
use crate::types::*;
use crate::Result;
use hdk::prelude::*;
use serde_json::Value;

/// Holochain対応Kernel
pub struct HolochainKernel {
    /// Mediator for actor selection
    mediator: HolochainMediator,
    /// Provenance service
    provenance: HolochainProvenance,
    /// Evolution engine
    evolution: HolochainEvolutionEngine,
}

impl HolochainKernel {
    /// 新しいKernelを作成
    pub fn new() -> Result<Self> {
        Ok(Self {
            mediator: HolochainMediator::new(),
            provenance: HolochainProvenance::new(),
            evolution: HolochainEvolutionEngine::new(),
        })
    }

    /// プロセスを実行
    pub async fn run_process(
        &mut self,
        process_id: &str,
        story_id: &str,
    ) -> Result<ProcessExecutionResult> {
        use crate::dht::{get_jsonld_entry, query_dht, resolve_cid};
        use crate::types::{ProcessEntry, StoryEntry};
        use kotoba_os::types::{Process, Story};
        use serde_json::json;

        // Storyを取得
        let story_entry_value = resolve_cid(story_id).await?;
        let story_entry: StoryEntry = serde_json::from_value(story_entry_value)
            .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
        
        let story = Story::from_value(story_entry.story.clone())
            .map_err(|e| crate::HolochainKotobasosError::Kernel(
                format!("Failed to parse story: {}", e)
            ))?;

        // Processを取得
        let process_entry_value = resolve_cid(process_id).await?;
        let process_entry: ProcessEntry = serde_json::from_value(process_entry_value)
            .map_err(|e| crate::HolochainKotobasosError::Serialization(e))?;
        
        let process = Process::from_value(process_entry.process.clone())
            .map_err(|e| crate::HolochainKotobasosError::Kernel(
                format!("Failed to parse process: {}", e)
            ))?;

        // Actorを選択（Mediatorを使用）
        let actor = self.mediator.select_actor(process_id).await?;

        // プロセスを実行
        // 実際のプロセス実行は、既存のkotoba-osのActorパターンを使用
        // ここでは簡易的な実装として、プロセスの出力をそのまま返す
        let output = json!({
            "process_id": process_id,
            "story_id": story_id,
            "executed_by": actor.id,
            "result": "success"
        });

        // Provenanceを記録
        let provenance_id = self
            .provenance
            .record(process_id, &actor, &output)
            .await?;

        Ok(ProcessExecutionResult {
            process_id: process_id.to_string(),
            success: true,
            output,
            provenance_id,
        })
    }

    /// Evolution Engineを実行
    pub async fn evolve(
        &mut self,
        provenance_id: &str,
        evolution_type: EvolutionType,
    ) -> Result<EvolutionResult> {
        self.evolution
            .evolve(provenance_id, evolution_type)
            .await
    }

    /// 進化履歴を取得
    pub async fn get_evolution_history(&self, story_id: &str) -> Result<Value> {
        self.evolution.get_history(story_id).await
    }
}

/// プロセス実行結果
#[derive(Debug, Clone)]
pub struct ProcessExecutionResult {
    pub process_id: String,
    pub success: bool,
    pub output: Value,
    pub provenance_id: String,
}

/// 進化結果
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub evolution_id: String,
    pub evolution_data: Value,
}

