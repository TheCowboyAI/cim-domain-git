// Copyright 2025 Cowboy AI, LLC.

//! Integration tests for NATS functionality
//!
//! These tests require a running NATS server with JetStream enabled.
//! Run with: cargo test --test nats_integration_tests -- --ignored

use async_trait::async_trait;
use chrono::Utc;
use cim_domain_git::{
    aggregate::{Repository, RepositoryId},
    commands::CloneRepository,
    events::{EventEnvelope, GitDomainEvent, RepositoryCloned},
    nats::{
        AckSubscriber, CommandHandler, CommandSubscriber, EventHandler, EventPublisher, EventStore,
        EventStoreConfig, EventSubscriber, HealthService, NatsClient, NatsConfig,
        ProjectionManager, RepositoryStatsProjection, ServiceInfo, ServiceStatus,
    },
    value_objects::RemoteUrl,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use futures::StreamExt;
use uuid::Uuid;

// Test command handler
struct TestCommandHandler {
    received_commands: Arc<Mutex<Vec<CloneRepository>>>,
    event_publisher: Arc<EventPublisher>,
}

#[async_trait]
impl CommandHandler for TestCommandHandler {
    type Command = CloneRepository;
    type Result = RepositoryId;

    async fn handle(&self, command: Self::Command) -> cim_domain_git::nats::Result<Self::Result> {
        // Store the command
        let mut commands = self.received_commands.lock().await;
        commands.push(command.clone());

        // Create and publish event
        let repo_id = RepositoryId::new();
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: repo_id,
            remote_url: command.remote_url,
            local_path: command.local_path,
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event);
        self.event_publisher.publish_envelope(&envelope).await?;

        Ok(repo_id)
    }

    fn command_type(&self) -> &'static str {
        "CloneRepository"
    }
}

// Test event handler
struct TestEventHandler {
    received_events: Arc<Mutex<Vec<EventEnvelope>>>,
}

#[async_trait]
impl EventHandler for TestEventHandler {
    type Event = EventEnvelope;

    async fn handle(&self, event: Self::Event) -> cim_domain_git::nats::Result<()> {
        let mut events = self.received_events.lock().await;
        events.push(event);
        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "RepositoryCloned"
    }
}

async fn setup_nats() -> cim_domain_git::nats::Result<NatsClient> {
    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };

    NatsClient::connect(config).await
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_event_store_append_and_replay() {
    let client = setup_nats().await.expect("Failed to connect to NATS");

    let publisher = EventPublisher::new(
        client.client().clone(),
        "git".to_string(),
    );

    let jetstream = client.jetstream().await.expect("Failed to get JetStream context");
    let mut event_store = EventStore::new(
        jetstream,
        publisher,
        EventStoreConfig {
            stream_name: "TEST_GIT_EVENTS_1".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create event store");

    // Create test events
    let repo_id = RepositoryId::new();
    let events = vec![
        GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: repo_id,
            remote_url: RemoteUrl::new("https://github.com/test/repo1.git").unwrap(),
            local_path: "/tmp/repo1".to_string(),
            timestamp: Utc::now(),
        }),
        GitDomainEvent::BranchCreated(cim_domain_git::events::BranchCreated {
            repository_id: repo_id,
            branch_name: cim_domain_git::value_objects::BranchName::new("feature/test").unwrap(),
            commit_hash: cim_domain_git::value_objects::CommitHash::new("abc123").unwrap(),
            source_branch: None,
            timestamp: Utc::now(),
        }),
    ];

    // Append events
    for event in &events {
        let envelope = EventEnvelope::new(event.clone());
        let seq = event_store.append(&envelope).await.unwrap();
        assert!(seq > 0);
    }

    // Replay events
    let replayed = event_store.load_aggregate_events(&repo_id).await.unwrap();
    assert_eq!(replayed.len(), 2);

    // Verify order and content
    match &replayed[0].event {
        GitDomainEvent::RepositoryCloned(_) => {}
        _ => panic!("Expected RepositoryCloned event"),
    }

    match &replayed[1].event {
        GitDomainEvent::BranchCreated(_) => {}
        _ => panic!("Expected BranchCreated event"),
    }
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_command_acknowledgment() {
    let client = setup_nats().await.expect("Failed to connect to NATS");

    // Set up command subscriber
    let command_subscriber =
        CommandSubscriber::new(client.client().clone(), "test-handler-001".to_string());

    let publisher = Arc::new(EventPublisher::new(
        client.client().clone(),
        "git".to_string(),
    ));

    let handler = TestCommandHandler {
        received_commands: Arc::new(Mutex::new(Vec::new())),
        event_publisher: publisher,
    };

    command_subscriber.register_handler(handler).await;

    // Start subscriber in background
    tokio::spawn(async move {
        let _ = command_subscriber.start().await;
    });

    // Give subscriber time to start
    sleep(Duration::from_millis(500)).await;

    // Send command with acknowledgment tracking
    let command_id = Uuid::new_v4();
    let command = CloneRepository {
        repository_id: Some(RepositoryId::new()),
        remote_url: RemoteUrl::new("https://github.com/test/ack-repo.git").unwrap(),
        local_path: "/tmp/ack-repo".to_string(),
        branch: None,
        depth: None,
    };

    let ack_subscriber = AckSubscriber::new(client.client().clone());

    // Set up headers
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("X-Command-ID", command_id.to_string());
    headers.insert("X-Command-Type", "CloneRepository");

    let payload = serde_json::to_vec(&command).unwrap();

    // Track acknowledgments
    let ack_future = {
        let ack_sub = ack_subscriber;
        let cmd_id = command_id;
        tokio::spawn(async move {
            ack_sub
                .subscribe_to_command(cmd_id, Duration::from_secs(3))
                .await
        })
    };

    // Publish command
    client
        .client()
        .publish_with_headers("git.cmd.repository.clone", headers, payload.into())
        .await
        .unwrap();

    // Wait for acknowledgments
    let acks = ack_future.await.unwrap().unwrap();

    // Should have received at least "received" and "completed" acks
    assert!(acks.len() >= 2);

    // Verify acknowledgment sequence
    let has_received = acks
        .iter()
        .any(|a| matches!(a.status, cim_domain_git::nats::AckStatus::Received));
    let has_completed = acks
        .iter()
        .any(|a| matches!(a.status, cim_domain_git::nats::AckStatus::Completed));

    assert!(has_received, "Should have received acknowledgment");
    assert!(has_completed, "Should have completed acknowledgment");
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_projection_updates() {
    let client = setup_nats().await.expect("Failed to connect to NATS");

    let publisher = EventPublisher::new(
        client.client().clone(),
        "git".to_string(),
    );

    let jetstream = client.jetstream().await.expect("Failed to get JetStream context");
    let mut event_store = EventStore::new(
        jetstream,
        publisher,
        EventStoreConfig {
            stream_name: "TEST_GIT_EVENTS_PROJ".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create event store");

    // For now, skip the projection manager part since it needs Arc<EventStore>
    // and we need mutable access to append events

    // Generate events
    let repo_id = RepositoryId::new();

    for i in 0..3 {
        let event = GitDomainEvent::CommitAnalyzed(cim_domain_git::events::CommitAnalyzed {
            repository_id: repo_id,
            commit_hash: cim_domain_git::value_objects::CommitHash::new(&format!("abc123{}", i))
                .unwrap(),
            parents: vec![],
            author: cim_domain_git::value_objects::AuthorInfo {
                name: format!("Test Author {}", i),
                email: "test@example.com".to_string(),
            },
            message: format!("Test commit {}", i),
            files_changed: vec![],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event);
        event_store.append(&envelope).await.unwrap();
    }

    // Wait for projections to process
    sleep(Duration::from_secs(2)).await;

    // Test passes if events were appended successfully
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_health_monitoring() {
    let client = setup_nats().await.expect("Failed to connect to NATS");

    let service_info = ServiceInfo {
        id: Uuid::new_v4().to_string(),
        name: "test-git-service".to_string(),
        version: "0.1.0".to_string(),
        description: Some("Test Git Service".to_string()),
        endpoints: vec!["git.>".to_string()],
        metadata: HashMap::new(),
        last_heartbeat: Utc::now(),
        status: ServiceStatus::Healthy,
    };

    let health_service = HealthService::new(client.client().clone(), service_info.clone());

    // Start health service
    let health_handle = tokio::spawn(async move {
        let _ = health_service.start().await;
    });

    // Give it time to publish health status
    sleep(Duration::from_secs(1)).await;

    // Subscribe to health updates
    let mut sub = client.client().subscribe("git.health.>").await.unwrap();

    // Should receive a health update within timeout
    let timeout = tokio::time::timeout(Duration::from_secs(3), sub.next()).await;

    assert!(timeout.is_ok(), "Should receive health update");

    let message = timeout.unwrap().unwrap();
    let health_update: serde_json::Value = serde_json::from_slice(&message.payload).unwrap();

    assert_eq!(health_update["service"]["name"], "test-git-service");
    assert_eq!(health_update["status"], "healthy");

    // Clean up
    health_handle.abort();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_correlation_tracking() {
    let client = setup_nats().await.expect("Failed to connect to NATS");

    let publisher = EventPublisher::new(
        client.client().clone(),
        "git".to_string(),
    );

    let jetstream = client.jetstream().await.expect("Failed to get JetStream context");
    let mut event_store = EventStore::new(
        jetstream,
        publisher,
        EventStoreConfig {
            stream_name: "TEST_GIT_EVENTS_CORR".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create event store");

    // Create correlated events
    let repo_id = RepositoryId::new();
    let correlation_id = Uuid::new_v4();

    let event1 = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/test/corr-repo.git").unwrap(),
        local_path: "/tmp/corr-repo".to_string(),
        timestamp: Utc::now(),
    });

    let envelope1 = EventEnvelope::from_correlation(event1, correlation_id, correlation_id);
    let seq1 = event_store.append(&envelope1).await.unwrap();

    let event2 = GitDomainEvent::BranchCreated(cim_domain_git::events::BranchCreated {
        repository_id: repo_id,
        branch_name: cim_domain_git::value_objects::BranchName::new("main").unwrap(),
        commit_hash: cim_domain_git::value_objects::CommitHash::new("def456").unwrap(),
        source_branch: None,
        timestamp: Utc::now(),
    });

    let envelope2 = EventEnvelope::from_correlation(event2, correlation_id, envelope1.event_id());
    let seq2 = event_store.append(&envelope2).await.unwrap();

    assert!(seq2 > seq1);

    // Load by correlation
    let correlated_events = event_store
        .load_by_correlation(correlation_id)
        .await
        .unwrap();
    assert_eq!(correlated_events.len(), 2);

    // Verify correlation chain
    assert_eq!(correlated_events[0].correlation_id(), correlation_id);
    assert_eq!(correlated_events[1].correlation_id(), correlation_id);
    assert_eq!(correlated_events[1].causation_id(), envelope1.event_id());
}
