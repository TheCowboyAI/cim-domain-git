// Copyright 2025 Cowboy AI, LLC.

//! Example demonstrating JetStream as an event store for event sourcing
//!
//! This example shows how to:
//! - Append events to JetStream
//! - Replay events to rebuild aggregate state
//! - Use durable consumers for projections
//! - Track consumer positions

use chrono::Utc;
use cim_domain_git::{
    aggregate::{Repository, RepositoryId},
    commands::{AnalyzeCommit, CloneRepository},
    events::{CommitAnalyzed, EventEnvelope, GitDomainEvent, RepositoryCloned},
    handlers::commands::CloneRepositoryHandler,
    nats::{
        ConsumerPosition, EventPublisher, EventStore, EventStoreConfig, NatsClient, NatsConfig,
    },
    value_objects::{AuthorInfo, CommitHash, FilePath, RemoteUrl},
};
use futures::StreamExt;
use std::error::Error;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to NATS
    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };

    let client = NatsClient::connect(config).await?;
    info!("Connected to NATS with JetStream");

    // Create event store (JetStream wrapper)
    let publisher = EventPublisher::new(client.client.clone(), "git".to_string());
    let event_store = EventStore::new(
        client.jetstream.clone(),
        publisher,
        EventStoreConfig::default(),
    )
    .await?;

    // Example 1: Append events to JetStream
    info!("=== Appending Events to JetStream ===");

    let repo_id = RepositoryId::new();

    // Create and append a RepositoryCloned event
    let clone_event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/example/repo.git")?,
        local_path: "/tmp/example-repo".to_string(),
        timestamp: Utc::now(),
    });

    let envelope1 = EventEnvelope::new(clone_event);
    let seq1 = event_store.append(&envelope1).await?;
    info!("Appended RepositoryCloned event at sequence: {}", seq1);

    // Create and append a CommitAnalyzed event
    let commit_event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
        repository_id: repo_id,
        commit_hash: CommitHash::new("abc123def456")?,
        parents: vec![],
        author: AuthorInfo {
            name: "Example Author".to_string(),
            email: "author@example.com".to_string(),
        },
        message: "Initial commit".to_string(),
        files_changed: vec![],
        commit_timestamp: Utc::now(),
        timestamp: Utc::now(),
    });

    let envelope2 = EventEnvelope::from_correlation(
        commit_event,
        envelope1.correlation_id(),
        envelope1.event_id(),
    );
    let seq2 = event_store.append(&envelope2).await?;
    info!("Appended CommitAnalyzed event at sequence: {}", seq2);

    // Example 2: Replay events to rebuild aggregate
    info!("\n=== Replaying Events from JetStream ===");

    let events = event_store.load_aggregate_events(&repo_id).await?;
    info!("Loaded {} events for repository {}", events.len(), repo_id);

    // Rebuild aggregate from events
    let mut repository = Repository::new(repo_id);
    for envelope in &events {
        match &envelope.event {
            GitDomainEvent::RepositoryCloned(e) => {
                info!("Replaying: Repository cloned from {}", e.remote_url);
                // Apply event to aggregate
            }
            GitDomainEvent::CommitAnalyzed(e) => {
                info!("Replaying: Commit {} analyzed", e.commit_hash);
                // Apply event to aggregate
            }
            _ => {}
        }
    }

    // Example 3: Create durable consumer for projections
    info!("\n=== Creating Durable Consumer ===");

    let consumer = event_store
        .create_durable_consumer("example_projection", None)
        .await?;

    info!("Created durable consumer 'example_projection'");

    // Process events with the consumer
    let mut messages = consumer.messages().await?;
    let mut processed_count = 0;

    info!("Processing events from durable consumer...");

    // Process a few messages
    while processed_count < 2 {
        if let Some(Ok(message)) = messages.next().await {
            if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                info!(
                    "Processing event {} (seq: {})",
                    envelope.event_type(),
                    message.info()?.stream_sequence
                );

                // Acknowledge the message
                message.ack().await?;
                processed_count += 1;

                // In a real projection, you would update read models here
            }
        }
    }

    // Example 4: Get events after a specific sequence
    info!("\n=== Getting Events After Sequence ===");

    let recent_events = event_store.get_events_after(seq1, Some(10)).await?;
    info!(
        "Found {} events after sequence {}",
        recent_events.len(),
        seq1
    );

    for envelope in &recent_events {
        info!(
            "  - {} at {}",
            envelope.event_type(),
            envelope.occurred_at()
        );
    }

    // Example 5: Stream info
    info!("\n=== Stream Information ===");

    let stream_info = event_store.info().await?;
    info!("Stream: {}", stream_info.name);
    info!("Total messages: {}", stream_info.messages);
    info!("Total bytes: {}", stream_info.bytes);
    info!(
        "Sequence range: {} - {}",
        stream_info.first_seq, stream_info.last_seq
    );
    info!("Active consumers: {}", stream_info.consumer_count);

    // Key points about JetStream as event store:
    info!("\n=== JetStream Event Store Summary ===");
    info!("✓ JetStream provides durable, ordered storage of events");
    info!("✓ Events are immutable once written");
    info!("✓ Consumers track their position for exactly-once processing");
    info!("✓ Stream replay enables aggregate rebuilding");
    info!("✓ Subject filtering allows efficient aggregate event loading");
    info!("✓ No additional event store abstraction needed - JetStream IS the event store!");

    Ok(())
}
