//! GraphStream engine for real-time graph processing
//!
//! Provides streaming provenance events and real-time pattern detection
//! for KotobaOS execution graphs.

use crate::provenance::Provenance;
use crate::types::ProvenanceEvent;
use crate::{KotobaOsError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Stream subscription handle
pub struct StreamSubscription {
    /// Stream ID
    pub stream_id: String,
    /// Receiver for events
    receiver: broadcast::Receiver<Value>,
}

impl StreamSubscription {
    /// Receive next event from stream
    pub async fn recv(&mut self) -> Result<Value> {
        self.receiver
            .recv()
            .await
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Stream receive error: {}", e)))
    }

    /// Try to receive event without blocking
    pub fn try_recv(&mut self) -> Result<Value> {
        self.receiver
            .try_recv()
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Stream try_recv error: {}", e)))
    }
}

/// GraphStream engine for real-time processing
pub struct GraphStream {
    /// Active stream subscriptions
    streams: Arc<RwLock<HashMap<String, broadcast::Sender<Value>>>>,
    /// Pattern detection patterns
    patterns: Vec<PatternDetector>,
    /// Buffer for recent events
    event_buffer: Arc<RwLock<Vec<ProvenanceEvent>>>,
    /// Buffer size limit
    buffer_limit: usize,
}

/// Pattern detector configuration
#[derive(Debug, Clone)]
pub struct PatternDetector {
    /// Pattern name
    pub name: String,
    /// SPARQL query for pattern detection
    pub query: String,
    /// Pattern threshold (minimum occurrences)
    pub threshold: usize,
}

impl GraphStream {
    /// Create a new GraphStream engine
    pub fn new(buffer_limit: usize) -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            patterns: Vec::new(),
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_limit,
        }
    }

    /// Create with default buffer size (1000 events)
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Subscribe to provenance events stream
    pub async fn subscribe(&self, stream_id: String) -> StreamSubscription {
        let mut streams = self.streams.write().await;
        
        let sender = streams.entry(stream_id.clone()).or_insert_with(|| {
            broadcast::channel(100).0
        }).clone();
        
        let receiver = sender.subscribe();
        
        StreamSubscription {
            stream_id,
            receiver,
        }
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: &ProvenanceEvent) -> Result<()> {
        // Convert event to JSON-LD
        let event_jsonld = serde_json::to_value(event)
            .map_err(|e| KotobaOsError::Other(anyhow::anyhow!("Failed to serialize event: {}", e)))?;

        // Add to buffer
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push(event.clone());
            
            // Maintain buffer size limit
            if buffer.len() > self.buffer_limit {
                buffer.remove(0);
            }
        }

        // Publish to all streams
        let streams = self.streams.read().await;
        let mut failed_streams = Vec::new();
        
        for (stream_id, sender) in streams.iter() {
            if sender.send(event_jsonld.clone()).is_err() {
                failed_streams.push(stream_id.clone());
            }
        }

        // Clean up failed streams
        if !failed_streams.is_empty() {
            let mut streams_write = self.streams.write().await;
            for stream_id in failed_streams {
                streams_write.remove(&stream_id);
            }
        }

        // Run pattern detection
        self.detect_patterns().await?;

        Ok(())
    }

    /// Add a pattern detector
    pub fn add_pattern(&mut self, pattern: PatternDetector) {
        self.patterns.push(pattern);
        info!("[GraphStream] Added pattern detector: {}", pattern.name);
    }

    /// Detect patterns in recent events
    async fn detect_patterns(&self) -> Result<()> {
        if self.patterns.is_empty() {
            return Ok(());
        }

        // Get recent events as JSON-LD
        let buffer = self.event_buffer.read().await;
        let events_jsonld = json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
            },
            "@graph": buffer.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect::<Vec<_>>()
        });

        drop(buffer);

        // Run each pattern detector
        #[cfg(feature = "reasoning")]
        {
            use kotoba_owl_reasoner::execute_sparql;
            
            for pattern in &self.patterns {
                match execute_sparql(&pattern.query, &events_jsonld).await {
                    Ok(result) => {
                        if let Some(graph) = result.get("@graph").and_then(|g| g.as_array()) {
                            if graph.len() >= pattern.threshold {
                                info!("[GraphStream] Pattern '{}' detected with {} matches (threshold: {})", 
                                    pattern.name, graph.len(), pattern.threshold);
                                
                                // Publish pattern detection event
                                let pattern_event = json!({
                                    "@type": "kotoba:PatternDetection",
                                    "kotoba:patternName": pattern.name,
                                    "kotoba:matchCount": graph.len(),
                                    "kotoba:matches": graph,
                                });
                                
                                let streams = self.streams.read().await;
                                for sender in streams.values() {
                                    let _ = sender.send(pattern_event.clone());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("[GraphStream] Pattern detection failed for '{}': {}", pattern.name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get recent events as JSON-LD
    pub async fn recent_events_jsonld(&self) -> Value {
        let buffer = self.event_buffer.read().await;
        json!({
            "@context": {
                "@vocab": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab",
                "kotoba": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld#vocab"
            },
            "@graph": buffer.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect::<Vec<_>>()
        })
    }

    /// Get stream statistics
    pub async fn stats(&self) -> HashMap<String, Value> {
        let streams = self.streams.read().await;
        let buffer = self.event_buffer.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("active_streams".to_string(), json!(streams.len()));
        stats.insert("buffered_events".to_string(), json!(buffer.len()));
        stats.insert("pattern_detectors".to_string(), json!(self.patterns.len()));
        
        stats
    }
}

impl Default for GraphStream {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProvenanceEvent;
    use chrono::Utc;

    fn create_test_event(id: &str) -> ProvenanceEvent {
        ProvenanceEvent {
            id: id.to_string(),
            type_: "kotoba:ProvenanceEvent".to_string(),
            context: None,
            was_generated_by: "process:1".to_string(),
            was_associated_with: Some("actor:1".to_string()),
            used: None,
            generated: None,
            ended_at_time: Utc::now().to_rfc3339(),
            additional: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_graph_stream_subscribe() {
        let stream = GraphStream::default();
        let mut subscription = stream.subscribe("test_stream".to_string()).await;
        
        // Publish an event
        let event = create_test_event("event:1");
        stream.publish(&event).await.unwrap();
        
        // Receive event
        let received = subscription.recv().await.unwrap();
        assert!(received.get("@id").is_some());
    }

    #[tokio::test]
    async fn test_graph_stream_buffer() {
        let stream = GraphStream::new(10);
        
        // Add more events than buffer limit
        for i in 0..15 {
            let event = create_test_event(&format!("event:{}", i));
            stream.publish(&event).await.unwrap();
        }
        
        let stats = stream.stats().await;
        assert_eq!(stats.get("buffered_events").and_then(|v| v.as_u64()), Some(10));
    }
}

