//! DHTベースのMediator実装
//!
//! DHTを利用したアクター検索と選択を行います。

use crate::dht::{get_entry_links, query_dht};
use crate::types::{ActorEntry, DhtQuery};
use crate::Result;
use hdk::prelude::*;
use serde_json::Value;

/// Holochain Mediator
pub struct HolochainMediator {
    /// ローカルキャッシュされたアクター
    local_actors: Vec<ActorEntry>,
}

impl HolochainMediator {
    /// 新しいMediatorを作成
    pub fn new() -> Self {
        Self {
            local_actors: Vec::new(),
        }
    }

    /// アクターを選択
    pub async fn select_actor(&self, process_id: &str) -> Result<ActorEntry> {
        // DHT上でアクターを検索
        let query = DhtQuery {
            entry_type: "Actor".to_string(),
            filters: json!({
                "process_id": process_id,
            }),
            pagination: None,
        };

        let results = query_dht(&query).await?;

        // 最初のアクターを返す（実際の実装では、より高度な選択ロジックを実装）
        if let Some((_, entry_value)) = results.first() {
            let actor_entry: ActorEntry = serde_json::from_value(entry_value.clone())?;
            Ok(actor_entry)
        } else {
            // ローカルアクターから検索
            if let Some(actor) = self.local_actors.first() {
                Ok(actor.clone())
            } else {
                Err(crate::HolochainKotobasosError::Actor(
                    "No actor found".to_string()
                ))
            }
        }
    }

    /// ローカルアクターを追加
    pub fn add_local_actor(&mut self, actor: ActorEntry) {
        self.local_actors.push(actor);
    }
}

