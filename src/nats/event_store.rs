// Copyright 2025 Cowboy AI, LLC.

//! JetStream event store for Git domain events
//!
//! This module provides a thin wrapper around NATS JetStream to use it as an event store
//! for event sourcing. JetStream provides the durability, ordering, and replay capabilities
//! needed for event sourcing.

use async_nats::jetstream::stream::Stream;
use async_nats::jetstream::{consumer::Consumer, Context as JetStreamContext};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

use super::{
    error::{NatsError, Result},
    publisher::EventPublisher,
};
use crate::aggregate::RepositoryId;
use crate::events::EventEnvelope;

/// Event store configuration
#[derive(Debug, Clone)]
pub struct EventStoreConfig {
    /// Stream name for events
    pub stream_name: String,

    /// Maximum age for events
    pub max_age: Duration,

    /// Maximum messages in the stream
    pub max_messages: i64,

    /// Number of replicas
    pub num_replicas: usize,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            stream_name: "GIT_EVENTS".to_string(),
            max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
            max_messages: 10_000_000,
            num_replicas: 1,
        }
    }
}

/// JetStream-based event store for Git domain events
///
/// This is a thin wrapper around JetStream that provides event sourcing capabilities.
/// JetStream itself IS the event store, providing:
/// - Durable, ordered storage of events
/// - Stream replay for aggregate rebuilding  
/// - Consumer checkpoints for processing position
/// - Subject-based filtering for aggregate events
pub struct EventStore {
    #[allow(dead_code)]
    jetstream: JetStreamContext,
    stream: Stream,
    publisher: EventPublisher,
    #[allow(dead_code)]
    config: EventStoreConfig,
}

impl EventStore {
    /// Create a new event store
    pub async fn new(
        jetstream: JetStreamContext,
        publisher: EventPublisher,
        config: EventStoreConfig,
    ) -> Result<Self> {
        // Ensure stream exists
        let stream = Self::ensure_stream(&jetstream, &config).await?;

        Ok(Self {
            jetstream,
            stream,
            publisher,
            config,
        })
    }

    /// Ensure the event stream exists with proper configuration
    async fn ensure_stream(
        jetstream: &JetStreamContext,
        config: &EventStoreConfig,
    ) -> Result<Stream> {
        use async_nats::jetstream::stream::{Config as StreamConfig, RetentionPolicy, StorageType};

        let stream_config = StreamConfig {
            name: config.stream_name.clone(),
            subjects: vec![format!("{}.event.>", super::subject::DOMAIN)],
            max_age: config.max_age,
            max_messages: config.max_messages,
            storage: StorageType::File,
            num_replicas: config.num_replicas,
            retention: RetentionPolicy::Limits,
            ..Default::default()
        };

        // Try to get existing stream first
        match jetstream.get_stream(&config.stream_name).await {
            Ok(stream) => {
                info!("Using existing event store stream: {}", config.stream_name);
                Ok(stream)
            }
            Err(_) => {
                // Stream doesn't exist, create it
                match jetstream.create_stream(stream_config).await {
                    Ok(stream) => {
                        info!("Created new event store stream: {}", config.stream_name);
                        Ok(stream)
                    }
                    Err(e) => Err(NatsError::Other(format!(
                        "Failed to create event stream: {}",
                        e
                    ))),
                }
            }
        }
    }

    /// Append an event to JetStream
    ///
    /// Events are published through the event publisher which adds proper headers
    /// and routes to the correct subject. JetStream automatically assigns a
    /// sequence number and persists the event.
    pub async fn append(&mut self, envelope: &EventEnvelope) -> Result<u64> {
        // Publish via the event publisher (which handles headers and routing)
        self.publisher.publish_envelope(envelope).await?;

        // Get the sequence number from the stream
        let info = self
            .stream
            .info()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get stream info: {}", e)))?;

        Ok(info.state.messages)
    }

    /// Append multiple events atomically
    pub async fn append_batch(&mut self, envelopes: &[EventEnvelope]) -> Result<Vec<u64>> {
        let mut sequences = Vec::with_capacity(envelopes.len());

        for envelope in envelopes {
            let seq = self.append(envelope).await?;
            sequences.push(seq);
        }

        Ok(sequences)
    }

    /// Load all events for an aggregate from JetStream
    ///
    /// Uses JetStream's replay capability to load all events for a specific aggregate.
    /// Events are filtered by checking the aggregate ID in the event metadata.
    pub async fn load_aggregate_events(
        &self,
        aggregate_id: &RepositoryId,
    ) -> Result<Vec<EventEnvelope>> {
        // Use ephemeral consumer to replay all events
        let _consumer_name = format!("aggregate_replay_{}", aggregate_id);
        let filter = format!("{}.event.>", super::subject::DOMAIN);

        // Create ephemeral consumer that starts from the beginning
        let consumer: Consumer<async_nats::jetstream::consumer::pull::Config> = self
            .stream
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                filter_subject: filter,
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
                ack_policy: async_nats::jetstream::consumer::AckPolicy::None,
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::Other(format!("Failed to create consumer: {}", e)))?;

        let mut events = Vec::new();
        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get messages: {}", e)))?;

        while let Some(Ok(message)) = messages.next().await {
            // Parse the envelope
            if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                // Check if it belongs to our aggregate
                if envelope.aggregate_id() == aggregate_id.to_string() {
                    events.push(envelope);
                }
            }
        }

        // Events are already ordered by JetStream sequence
        Ok(events)
    }

    /// Load events by correlation ID
    pub async fn load_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<EventEnvelope>> {
        let filter = format!("{}.event.>", super::subject::DOMAIN);

        // Create a consumer
        let consumer: Consumer<async_nats::jetstream::consumer::pull::Config> = self
            .stream
            .get_or_create_consumer(
                "correlation_loader",
                async_nats::jetstream::consumer::pull::Config {
                    filter_subject: filter,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| NatsError::Other(format!("Failed to create consumer: {}", e)))?;

        let mut events = Vec::new();
        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get messages: {}", e)))?;

        while let Some(Ok(message)) = messages.next().await {
            // Check correlation ID in headers first
            if let Some(headers) = &message.headers {
                if let Some(corr_id) = headers.get("X-Correlation-ID") {
                    if corr_id.as_str() == correlation_id.to_string() {
                        if let Ok(envelope) =
                            serde_json::from_slice::<EventEnvelope>(&message.payload)
                        {
                            events.push(envelope);
                        }
                    }
                }
            }

            // Acknowledge the message
            let _ = message.ack().await;
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.occurred_at());

        Ok(events)
    }

    /// Get events after a specific sequence number
    ///
    /// Useful for projections that need to catch up from a known position
    pub async fn get_events_after(
        &self,
        start_sequence: u64,
        limit: Option<usize>,
    ) -> Result<Vec<EventEnvelope>> {
        let consumer: Consumer<async_nats::jetstream::consumer::pull::Config> = self
            .stream
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::ByStartSequence {
                    start_sequence: start_sequence + 1,
                },
                ack_policy: async_nats::jetstream::consumer::AckPolicy::None,
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::Other(format!("Failed to create consumer: {}", e)))?;

        let mut events = Vec::new();
        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get messages: {}", e)))?;

        let max_events = limit.unwrap_or(usize::MAX);

        while events.len() < max_events {
            match messages.next().await {
                Some(Ok(message)) => {
                    if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload)
                    {
                        events.push(envelope);
                    }
                }
                _ => break,
            }
        }

        Ok(events)
    }

    /// Create a durable consumer for processing events
    ///
    /// This is used by projections and event handlers that need to process
    /// events and track their position
    pub async fn create_durable_consumer(
        &self,
        consumer_name: &str,
        filter_subject: Option<String>,
    ) -> Result<Consumer<async_nats::jetstream::consumer::pull::Config>> {
        let filter =
            filter_subject.unwrap_or_else(|| format!("{}.event.>", super::subject::DOMAIN));

        let consumer = self
            .stream
            .get_or_create_consumer(
                consumer_name,
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(consumer_name.to_string()),
                    filter_subject: filter,
                    ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| NatsError::Other(format!("Failed to create consumer: {}", e)))?;

        info!("Created durable consumer: {}", consumer_name);

        Ok(consumer)
    }

    /// Get stream info
    pub async fn info(&mut self) -> Result<StreamInfo> {
        let info = self
            .stream
            .info()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get stream info: {}", e)))?;

        Ok(StreamInfo {
            name: info.config.name.clone(),
            messages: info.state.messages,
            bytes: info.state.bytes,
            first_seq: info.state.first_sequence,
            last_seq: info.state.last_sequence,
            consumer_count: info.state.consumer_count,
        })
    }
}

/// Consumer position tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerPosition {
    /// Consumer name
    pub consumer_name: String,

    /// Last processed sequence
    pub last_sequence: u64,

    /// Last acknowledgment time
    pub last_ack_time: chrono::DateTime<chrono::Utc>,
}

/// Stream information
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Stream name
    pub name: String,

    /// Total messages
    pub messages: u64,

    /// Total bytes
    pub bytes: u64,

    /// First sequence number
    pub first_seq: u64,

    /// Last sequence number
    pub last_seq: u64,

    /// Number of consumers
    pub consumer_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::RepositoryId;
    use crate::events::{EventEnvelopeBuilder, GitDomainEvent, RepositoryCloned};
    use crate::value_objects::RemoteUrl;

    #[tokio::test]
    #[ignore = "requires NATS server with JetStream"]
    async fn test_event_store_append() {
        // This test requires a running NATS server with JetStream enabled
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let jetstream = async_nats::jetstream::new(client.clone());
        let publisher = EventPublisher::new(client, "git".to_string());

        let config = EventStoreConfig {
            stream_name: "TEST_GIT_EVENTS".to_string(),
            ..Default::default()
        };

        let mut store = EventStore::new(jetstream, publisher, config).await.unwrap();

        // Create test event
        let repo_id = RepositoryId::new();
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: repo_id,
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/test".to_string(),
            timestamp: chrono::Utc::now(),
        });

        let envelope = EventEnvelope::new(event);

        // Append event
        let seq = store.append(&envelope).await.unwrap();
        assert!(seq > 0);

        // Load events
        let events = store.load_aggregate_events(&repo_id).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id(), envelope.event_id());
    }
}
