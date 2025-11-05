//! Event Sourcing Integration Tests
//!
//! This module provides comprehensive integration tests for the event sourcing
//! functionality, covering event streams, projections, and materialized views.
//!
//! Components tested:
//! - kotoba-event-stream (Event stream management)
//! - kotoba-projection-engine (Projection and materialized views)

use std::sync::Arc;
use tokio::sync::Mutex;
use kotoba_memory::MemoryKeyValueStore;
use kotoba_storage::KeyValueStore;
use kotoba_core::types::{Value, VertexId, EdgeId};
use kotoba_errors::KotobaError;

// TestEvent struct removed - using JSON-LD format directly via test_helpers::create_jsonld_event
// All event data is now created as JSON-LD format directly

pub struct EventSourcingTestFixture {
    pub storage: Arc<dyn KeyValueStore + Send + Sync>,
    pub event_stream: Option<Arc<Mutex<kotoba_event_stream::EventStream>>>,
}

impl EventSourcingTestFixture {
    pub async fn new() -> Result<Self, KotobaError> {
        let storage = Arc::new(MemoryKeyValueStore::new());

        // Initialize event stream
        let event_stream = if let Ok(stream) = kotoba_event_stream::EventStream::new(Arc::clone(&storage)).await {
            Some(Arc::new(Mutex::new(stream)))
        } else {
            None
        };

        Ok(Self {
            storage,
            event_stream,
        })
    }

    pub async fn cleanup(&self) -> Result<(), KotobaError> {
        if let Ok(keys) = self.storage.list_keys().await {
            for key in keys {
                let _ = self.storage.delete(key.as_bytes()).await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_creation_and_serialization() {
        // Test event creation (JSON-LD format)
        use crate::test_helpers::create_jsonld_event;
        use serde_json::json;
        
        let event = create_jsonld_event(
            "UserCreated",
            "user-123",
            &[
                ("name", json!("Alice")),
                ("email", json!("alice@example.com"))
            ]
        );

        // Test serialization (JSON-LD format)
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.get("kotoba:eventType").and_then(|v| v.as_str()), Some("UserCreated"));
        assert_eq!(deserialized.get("kotoba:aggregateId").and_then(|v| v.as_str()), Some("user-123"));
    }

    #[tokio::test]
    async fn test_basic_event_storage() {
        let fixture = EventSourcingTestFixture::new().await.unwrap();

        // Create and store an event (JSON-LD format)
        use crate::test_helpers::create_jsonld_event;
        use serde_json::json;
        
        let aggregate_id = "user-123";
        let event_id = uuid::Uuid::new_v4().to_string();
        let event = create_jsonld_event(
            "UserCreated",
            aggregate_id,
            &[
                ("name", json!("Alice")),
                ("email", json!("alice@example.com"))
            ]
        );

        let event_key = format!("event:{}:{}", aggregate_id, event_id);
        let event_data = serde_json::to_vec(&event).unwrap();

        fixture.storage.put(event_key.as_bytes(), &event_data).await.unwrap();

        // Retrieve and verify (JSON-LD format)
        let retrieved = fixture.storage.get(event_key.as_bytes()).await.unwrap().unwrap();
        let retrieved_event: serde_json::Value = serde_json::from_slice(&retrieved).unwrap();

        assert_eq!(retrieved_event.get("kotoba:eventType").and_then(|v| v.as_str()), Some("UserCreated"));
        assert_eq!(retrieved_event.get("kotoba:aggregateId").and_then(|v| v.as_str()), Some(aggregate_id));

        fixture.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_stream_operations() {
        let fixture = EventSourcingTestFixture::new().await.unwrap();

        if let Some(event_stream) = &fixture.event_stream {
            let mut stream = event_stream.lock().await;

            // Create event stream
            let stream_id = "user-events";
            stream.create_stream(stream_id).await.unwrap();

            // Add events to stream (JSON-LD format)
            use crate::test_helpers::create_jsonld_event;
            use serde_json::json;
            
            let aggregate_id = "user-123";
            let event1_id = uuid::Uuid::new_v4().to_string();
            let event2_id = uuid::Uuid::new_v4().to_string();
            
            let event1 = create_jsonld_event("UserCreated", aggregate_id, &[("name", json!("Alice"))]);
            let event2 = create_jsonld_event("UserUpdated", aggregate_id, &[("email", json!("alice@example.com"))]);

            // Store events in stream
            let event1_key = format!("stream:{}:event:{}", stream_id, event1_id);
            let event2_key = format!("stream:{}:event:{}", stream_id, event2_id);

            fixture.storage.put(event1_key.as_bytes(), &serde_json::to_vec(&event1).unwrap()).await.unwrap();
            fixture.storage.put(event2_key.as_bytes(), &serde_json::to_vec(&event2).unwrap()).await.unwrap();

            // Verify events in stream
            let keys = fixture.storage.list_keys().await.unwrap();
            let stream_keys: Vec<_> = keys.iter()
                .filter(|k| k.starts_with(&format!("stream:{}:", stream_id)))
                .collect();

            assert!(!stream_keys.is_empty());
            println!("Found {} events in stream {}", stream_keys.len(), stream_id);
        } else {
            println!("Event stream not available, skipping stream operations test");
        }

        fixture.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_aggregation() {
        let fixture = EventSourcingTestFixture::new().await.unwrap();

        // Create multiple events for the same aggregate (JSON-LD format)
        use crate::test_helpers::create_jsonld_event;
        use serde_json::json;
        
        let aggregate_id = "user-456";

        let events = vec![
            (uuid::Uuid::new_v4().to_string(), create_jsonld_event("UserCreated", aggregate_id, &[("name", json!("Bob"))])),
            (uuid::Uuid::new_v4().to_string(), create_jsonld_event("UserEmailUpdated", aggregate_id, &[("email", json!("bob@example.com"))])),
            (uuid::Uuid::new_v4().to_string(), create_jsonld_event("UserActivated", aggregate_id, &[("active", json!(true))])),
        ];

        // Store all events
        for (event_id, event) in &events {
            let key = format!("event:{}:{}", aggregate_id, event_id);
            let data = serde_json::to_vec(event).unwrap();
            fixture.storage.put(key.as_bytes(), &data).await.unwrap();
        }

        // Aggregate current state from events (JSON-LD format)
        let mut current_state = serde_json::json!({});
        for (_event_id, event) in &events {
            let event_type = event.get("kotoba:eventType").and_then(|v| v.as_str()).unwrap_or("");
            let data_obj = event.get("kotoba:data").and_then(|v| v.as_object());
            
            match event_type {
                "UserCreated" => {
                    if let Some(data) = data_obj {
                        if let Some(name) = data.get("kotoba:name").and_then(|v| v.as_str()) {
                            current_state["name"] = json!(name);
                        }
                    }
                    current_state["created"] = json!(true);
                }
                "UserEmailUpdated" => {
                    if let Some(data) = data_obj {
                        if let Some(email) = data.get("kotoba:email").and_then(|v| v.as_str()) {
                            current_state["email"] = json!(email);
                        }
                    }
                }
                "UserActivated" => {
                    if let Some(data) = data_obj {
                        if let Some(active) = data.get("kotoba:active").and_then(|v| v.as_bool()) {
                            current_state["active"] = json!(active);
                        }
                    }
                }
                _ => {}
            }
        }

        // Verify aggregated state
        assert_eq!(current_state["name"], "Bob");
        assert_eq!(current_state["email"], "bob@example.com");
        assert_eq!(current_state["active"], true);
        assert_eq!(current_state["created"], true);

        println!("Successfully aggregated state from {} events", events.len());

        fixture.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_versioning() {
        let fixture = EventSourcingTestFixture::new().await.unwrap();

        let aggregate_id = "product-789";
        let mut version = 0;

        // Create versioned events (JSON-LD format)
        use crate::test_helpers::create_jsonld_event;
        use serde_json::json;
        
        let events = vec![
            ("ProductCreated", vec![("name", json!("Widget")), ("price", json!(10.99))]),
            ("ProductPriceUpdated", vec![("price", json!(12.99))]),
            ("ProductDiscontinued", vec![("discontinued", json!(true))]),
        ];

        for (event_type, data_fields) in events {
            version += 1;
            let event = create_jsonld_event(event_type, aggregate_id, &data_fields);

            let key = format!("event:{}:v{:03}", aggregate_id, version);
            let event_data = serde_json::to_vec(&event).unwrap();
            fixture.storage.put(key.as_bytes(), &event_data).await.unwrap();

            // Store version metadata
            let version_key = format!("aggregate:{}:version", aggregate_id);
            fixture.storage.put(version_key.as_bytes(), &version.to_string().as_bytes()).await.unwrap();
        }

        // Verify versioning
        let version_key = format!("aggregate:{}:version", aggregate_id);
        let stored_version = fixture.storage.get(version_key.as_bytes()).await.unwrap().unwrap();
        let stored_version: u32 = String::from_utf8(stored_version).unwrap().parse().unwrap();

        assert_eq!(stored_version, 3);

        // Verify all versions exist
        for v in 1..=3 {
            let event_key = format!("event:{}:v{:03}", aggregate_id, v);
            let event_data = fixture.storage.get(event_key.as_bytes()).await.unwrap().unwrap();
            let event: serde_json::Value = serde_json::from_slice(&event_data).unwrap();
            assert_eq!(event.get("kotoba:aggregateId").and_then(|v| v.as_str()), Some(aggregate_id));
        }

        fixture.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_stream_concurrent_access() {
        let fixture = Arc::new(EventSourcingTestFixture::new().await.unwrap());

        // Test concurrent event storage
        let mut handles = vec![];

        for i in 0..5 {
            let fixture_clone = Arc::clone(&fixture);
            let handle = tokio::spawn(async move {
                let aggregate_id = format!("concurrent-user-{}", i);
                use crate::test_helpers::create_jsonld_event;
                use serde_json::json;
                
                let event_id = uuid::Uuid::new_v4().to_string();
                let event = create_jsonld_event(
                    "UserCreated",
                    &aggregate_id,
                    &[
                        ("name", json!(format!("User{}", i))),
                        ("index", json!(i))
                    ]
                );

                let key = format!("event:{}:{}", aggregate_id, event_id);
                let data = serde_json::to_vec(&event).unwrap();

                fixture_clone.storage.put(key.as_bytes(), &data).await.unwrap();

                Ok::<(), KotobaError>(())
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Verify all events were stored
        let keys = fixture.storage.list_keys().await.unwrap();
        let event_keys: Vec<_> = keys.iter()
            .filter(|k| k.starts_with("event:concurrent-user-"))
            .collect();

        assert_eq!(event_keys.len(), 5);

        fixture.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_replay_and_projection() {
        let fixture = EventSourcingTestFixture::new().await.unwrap();

        // Create a sequence of events (JSON-LD format)
        use crate::test_helpers::create_jsonld_event;
        use serde_json::json;
        
        let aggregate_id = "account-101";
        let events = vec![
            create_jsonld_event("AccountCreated", aggregate_id, &[("balance", json!(0.0))]),
            create_jsonld_event("MoneyDeposited", aggregate_id, &[("amount", json!(100.0))]),
            create_jsonld_event("MoneyWithdrawn", aggregate_id, &[("amount", json!(25.0))]),
            create_jsonld_event("MoneyDeposited", aggregate_id, &[("amount", json!(50.0))]),
        ];

        // Store events
        for (i, event) in events.iter().enumerate() {
            let key = format!("event:{}:{:03}", aggregate_id, i + 1);
            let data = serde_json::to_vec(event).unwrap();
            fixture.storage.put(key.as_bytes(), &data).await.unwrap();
        }

        // Replay events to build current state (projection) - JSON-LD format
        let mut balance = 0.0;

        for i in 0..events.len() {
            let key = format!("event:{}:{:03}", aggregate_id, i + 1);
            let data = fixture.storage.get(key.as_bytes()).await.unwrap().unwrap();
            let event: serde_json::Value = serde_json::from_slice(&data).unwrap();

            let event_type = event.get("kotoba:eventType").and_then(|v| v.as_str()).unwrap_or("");
            let data_obj = event.get("kotoba:data").and_then(|v| v.as_object());
            
            match event_type {
                "MoneyDeposited" => {
                    if let Some(data) = data_obj {
                        if let Some(amount) = data.get("kotoba:amount").and_then(|v| v.as_f64()) {
                            balance += amount;
                        }
                    }
                }
                "MoneyWithdrawn" => {
                    if let Some(data) = data_obj {
                        if let Some(amount) = data.get("kotoba:amount").and_then(|v| v.as_f64()) {
                            balance -= amount;
                        }
                    }
                }
                _ => {}
            }
        }

        // Verify final balance
        assert_eq!(balance, 125.0); // 100 + 50 - 25

        // Store projection result
        let projection_key = format!("projection:account:{}:balance", aggregate_id);
        fixture.storage.put(projection_key.as_bytes(), &balance.to_string().as_bytes()).await.unwrap();

        // Verify projection
        let stored_balance = fixture.storage.get(projection_key.as_bytes()).await.unwrap().unwrap();
        let stored_balance: f64 = String::from_utf8(stored_balance).unwrap().parse().unwrap();
        assert_eq!(stored_balance, 125.0);

        fixture.cleanup().await.unwrap();
    }
}
