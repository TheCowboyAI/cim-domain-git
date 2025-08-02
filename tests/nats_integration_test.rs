// Copyright 2025 Cowboy AI, LLC.

//! Integration tests for NATS functionality
//!
//! Note: These tests require a running NATS server.
//! Run with: NATS_URL=nats://localhost:4222 cargo test --test nats_integration_test -- --nocapture

use chrono::Utc;
use cim_domain_git::{
    aggregate::RepositoryId,
    events::{CommitAnalyzed, FileChangeInfo, FileChangeType, GitDomainEvent, RepositoryCloned},
    nats::{
        subject::{CommandAction, EventAction, GitSubject},
        CommandHandler, CommandSubscriber, EventHandler, EventSubscriber,
    },
    value_objects::{AuthorInfo, CommitHash, FilePath, RemoteUrl},
    EventPublisher, NatsClient, NatsConfig,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_nats_connection() {
    let config = NatsConfig::from_env().unwrap_or_default();
    let client = NatsClient::connect(config).await.unwrap();
    assert!(client.is_connected());

    // Test flush
    client.flush().await.unwrap();

    // Test graceful shutdown
    client.drain().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_event_publishing() {
    let config = NatsConfig::from_env().unwrap_or_default();
    let client = NatsClient::connect(config).await.unwrap();

    let publisher = EventPublisher::new(client.client().clone(), "git".to_string());

    // Create test event
    let repo_id = RepositoryId::new();
    let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
        local_path: "/tmp/test-repo".to_string(),
        timestamp: Utc::now(),
    });

    // Publish event
    publisher.publish_event(&event).await.unwrap();

    // Clean up
    client.drain().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_event_subscription() {
    let config = NatsConfig::from_env().unwrap_or_default();
    let client = NatsClient::connect(config).await.unwrap();

    // Create channel to receive events
    let (tx, mut rx) = mpsc::channel(10);

    // Set up event subscriber
    let subscriber = EventSubscriber::new(client.client().clone());
    subscriber
        .register_handler(TestEventHandler { sender: tx })
        .await;

    // Start subscriber in background
    let handle = tokio::spawn(async move {
        let _ = subscriber.start().await;
    });

    // Give subscriber time to set up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Publish test event
    let publisher = EventPublisher::new(client.client().clone(), "git".to_string());
    let repo_id = RepositoryId::new();
    let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
        local_path: "/tmp/test-repo".to_string(),
        timestamp: Utc::now(),
    });

    publisher.publish_event(&event).await.unwrap();

    // Wait for event to be received
    tokio::time::timeout(tokio::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Clean up
    handle.abort();
    client.drain().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_command_handling() {
    let config = NatsConfig::from_env().unwrap_or_default();
    let client = NatsClient::connect(config).await.unwrap();

    // Set up command subscriber
    let subscriber = CommandSubscriber::new(client.client().clone());
    subscriber.register_handler(TestCommandHandler).await;

    // Start subscriber in background
    let handle = tokio::spawn(async move {
        let _ = subscriber.start().await;
    });

    // Give subscriber time to set up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Send command and wait for response
    let subject = GitSubject::command(CommandAction::CloneRepository).to_string();
    let command = serde_json::json!({
        "repository_id": Uuid::new_v4().to_string(),
        "remote_url": "https://github.com/test/repo.git",
        "local_path": "/tmp/test-repo"
    });

    let mut headers = async_nats::HeaderMap::new();
    headers.insert("X-Command-Type", "CloneRepository");

    let response = client
        .client()
        .request_with_headers(
            subject,
            headers,
            serde_json::to_vec(&command).unwrap().into(),
        )
        .await
        .unwrap();

    let result: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
    assert_eq!(result["status"], "accepted");

    // Clean up
    handle.abort();
    client.drain().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_jetstream_integration() {
    let mut config = NatsConfig::from_env().unwrap_or_default();
    config.jetstream.enabled = true;
    config.jetstream.event_stream = "TEST_GIT_EVENTS".to_string();

    let client = NatsClient::connect(config).await.unwrap();

    // Get JetStream context
    let jetstream = client.jetstream().await.unwrap();

    // Publish to stream
    let subject = GitSubject::event(EventAction::RepositoryCloned).to_string();
    let event = serde_json::json!({
        "event_type": "RepositoryCloned",
        "repository_id": Uuid::new_v4().to_string(),
        "timestamp": Utc::now().to_rfc3339()
    });

    jetstream
        .publish(subject, serde_json::to_vec(&event).unwrap().into())
        .await
        .unwrap();

    // Clean up - delete test stream
    let _ = jetstream.delete_stream("TEST_GIT_EVENTS").await;
    client.drain().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_subject_routing() {
    // Test command subjects
    assert_eq!(
        GitSubject::command(CommandAction::CloneRepository).to_string(),
        "git.cmd.repository.clone"
    );

    // Test event subjects
    assert_eq!(
        GitSubject::event(EventAction::RepositoryCloned).to_string(),
        "git.event.repository.cloned"
    );

    // Test wildcards
    assert_eq!(
        GitSubject::wildcard(crate::nats::subject::MessageType::Event),
        "git.event.>"
    );
}

// Test event handler
struct TestEventHandler {
    sender: mpsc::Sender<()>,
}

#[async_trait::async_trait]
impl EventHandler for TestEventHandler {
    type Event = serde_json::Value;

    async fn handle(&self, _event: Self::Event) -> cim_domain_git::nats::Result<()> {
        self.sender.send(()).await.unwrap();
        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "RepositoryCloned"
    }
}

// Test command handler
struct TestCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for TestCommandHandler {
    type Command = serde_json::Value;
    type Result = serde_json::Value;

    async fn handle(&self, _command: Self::Command) -> cim_domain_git::nats::Result<Self::Result> {
        Ok(serde_json::json!({
            "status": "accepted",
            "message": "Command received"
        }))
    }

    fn command_type(&self) -> &'static str {
        "CloneRepository"
    }
}
