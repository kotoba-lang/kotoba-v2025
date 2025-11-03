//! Holochain Zome実装
//!
//! Holochainのzome関数としてKernel機能を公開します。

use crate::dht::{get_jsonld_entry, store_jsonld_entry};
use crate::kernel::HolochainKernel;
use crate::types::*;
use crate::Result;
use hdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// 新しいストーリー（プロセスネットワーク）を作成
#[hdk_extern]
pub async fn create_story(story_json: Value) -> ExternResult<EntryHash> {
    // Storyエントリを作成
    let author = agent_info()?.agent_latest_pubkey();
    let story_entry = StoryEntry {
        id: crate::utils::jsonld_to_cid(&story_json)
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?,
        story: story_json.clone(),
        created_at: chrono::Utc::now().timestamp(),
        author,
    };

    let entry_value = serde_json::to_value(&story_entry)
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    store_jsonld_entry("Story", &entry_value)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))
}

/// プロセスを実行
#[hdk_extern]
pub async fn run_process(input: RunProcessInput) -> ExternResult<RunProcessOutput> {
    // Kernelを使用してプロセスを実行
    let mut kernel = HolochainKernel::new()
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    // プロセスを実行
    let result = kernel
        .run_process(&input.process_id, &input.story_id)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    Ok(RunProcessOutput {
        success: true,
        result: ProcessExecutionResultValue {
            process_id: result.process_id.clone(),
            success: result.success,
            output: result.output.clone(),
            provenance_id: result.provenance_id.clone(),
        },
        provenance_id: result.provenance_id,
    })
}

/// アクターを登録
#[hdk_extern]
pub async fn register_actor(input: RegisterActorInput) -> ExternResult<EntryHash> {
    let agent = agent_info()?.agent_latest_pubkey();
    
    let actor_entry = ActorEntry {
        id: input.actor_id,
        capability: input.capability,
        agent,
        metadata: input.metadata.unwrap_or_else(|| json!({})),
        registered_at: chrono::Utc::now().timestamp(),
    };

    let entry_value = serde_json::to_value(&actor_entry)
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    store_jsonld_entry("Actor", &entry_value)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))
}

/// 実行履歴を取得
#[hdk_extern]
pub async fn get_provenance(input: GetProvenanceInput) -> ExternResult<Value> {
    // Provenanceエントリを取得
    let provenance_entry = get_jsonld_entry(&input.provenance_hash)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    Ok(provenance_entry)
}

/// Evolution Engine実行
#[hdk_extern]
pub async fn evolve(input: EvolveInput) -> ExternResult<EvolveOutput> {
    let mut kernel = HolochainKernel::new()
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    // Evolution Engineを実行
    let evolution_result = kernel
        .evolve(&input.provenance_id, input.evolution_type)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    Ok(EvolveOutput {
        evolution_id: evolution_result.evolution_id,
        evolution_data: evolution_result.evolution_data,
    })
}

/// 進化履歴を取得
#[hdk_extern]
pub async fn get_evolution_history(input: GetEvolutionHistoryInput) -> ExternResult<Value> {
    let kernel = HolochainKernel::new()
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    let history = kernel
        .get_evolution_history(&input.story_id)
        .await
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    Ok(history)
}

/// Zome関数の入力/出力型定義

#[derive(Serialize, Deserialize, Debug)]
pub struct RunProcessInput {
    pub process_id: String,
    pub story_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunProcessOutput {
    pub success: bool,
    pub result: ProcessExecutionResultValue,
    pub provenance_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessExecutionResultValue {
    pub process_id: String,
    pub success: bool,
    pub output: Value,
    pub provenance_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterActorInput {
    pub actor_id: String,
    pub capability: String,
    pub metadata: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProvenanceInput {
    pub provenance_hash: EntryHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvolveInput {
    pub provenance_id: String,
    pub evolution_type: EvolutionType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvolveOutput {
    pub evolution_id: String,
    pub evolution_data: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetEvolutionHistoryInput {
    pub story_id: String,
}

