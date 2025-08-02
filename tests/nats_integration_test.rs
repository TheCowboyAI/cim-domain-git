// Copyright 2025 Cowboy AI, LLC.

//! Integration tests for NATS functionality
//!
//! Note: These tests require a running NATS server.
//! Run with: NATS_URL=nats://localhost:4222 cargo test --test nats_integration_test -- --nocapture

use chrono::Utc;
use cim_domain_git::{
    aggregate::RepositoryId,
    events::{GitDomainEvent, RepositoryCloned},
    nats::{
        subject::{CommandAction, EventAction, GitSubject, MessageType},
        CommandHandler, EventHandler, EventSubscriber,
    },
    value_objects::RemoteUrl,
    EventPublisher, NatsClient, NatsConfig,
};
use futures::StreamExt;
use tokio::sync::mpsc;
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
    client.close().await.unwrap();
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
    client.close().await.unwrap();
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
    client.close().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_command_handling() {
    let config = NatsConfig::from_env().unwrap_or_default();
    let client = NatsClient::connect(config).await.unwrap();

    // For this test, we'll verify command publishing works
    // In production, the command handler runs in a separate service
    let subject = GitSubject::command(CommandAction::CloneRepository).to_string();
    let command = serde_json::json!({
        "repository_id": Uuid::new_v4().to_string(),
        "remote_url": "https://github.com/test/repo.git",
        "local_path": "/tmp/test-repo"
    });

    let mut headers = async_nats::HeaderMap::new();
    headers.insert("X-Command-Type", "CloneRepository");
    headers.insert("X-Command-ID", Uuid::new_v4().to_string());

    // Test that we can publish a command
    client
        .client()
        .publish_with_headers(
            subject.clone(),
            headers,
            serde_json::to_vec(&command).unwrap().into(),
        )
        .await
        .unwrap();

    // Verify the command was published by subscribing and receiving it
    let mut sub = client.client().subscribe(subject).await.unwrap();
    
    // Publish again
    let mut headers2 = async_nats::HeaderMap::new();
    headers2.insert("X-Command-Type", "CloneRepository");
    client
        .client()
        .publish_with_headers(
            GitSubject::command(CommandAction::CloneRepository).to_string(),
            headers2,
            serde_json::to_vec(&command).unwrap().into(),
        )
        .await
        .unwrap();
    
    // Should receive the message
    let msg = tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        sub.next()
    )
    .await
    .expect("Should receive message")
    .expect("Should have message");
    
    assert_eq!(msg.subject.as_str(), GitSubject::command(CommandAction::CloneRepository).to_string());
    
    client.close().await.unwrap();
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_jetstream_integration() {
    let mut config = NatsConfig::from_env().unwrap_or_default();
    config.jetstream.enabled = true;
    // Use a unique stream name to avoid conflicts
    let stream_name = format!("TEST_GIT_EVENTS_{}", Uuid::new_v4().to_string().replace("-", ""));
    config.jetstream.event_stream = stream_name.clone();

    let client = NatsClient::connect(config).await.unwrap();

    // Get JetStream context
    let jetstream = client.jetstream().await.unwrap();

    // Create the stream first
    let stream_config = async_nats::jetstream::stream::Config {
        name: stream_name.clone(),
        subjects: vec!["git.event.>".to_string()],
        ..Default::default()
    };
    
    // Create stream
    let _ = jetstream.create_stream(stream_config).await.unwrap();

    // Now publish to stream
    let subject = GitSubject::event(EventAction::RepositoryCloned).to_string();
    let event = serde_json::json!({
        "event_type": "RepositoryCloned",
        "repository_id": Uuid::new_v4().to_string(),
        "timestamp": Utc::now().to_rfc3339()
    });

    let ack = jetstream
        .publish(subject, serde_json::to_vec(&event).unwrap().into())
        .await
        .unwrap()
        .await
        .unwrap();
    
    // Verify the message was stored
    assert!(ack.sequence > 0);
    
    // Get stream to verify
    let mut stream = jetstream.get_stream(&stream_name).await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 1);

    // Clean up - delete test stream
    let _ = jetstream.delete_stream(&stream_name).await;
    
    // Flush any pending messages before closing
    let _ = client.flush().await;
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
        GitSubject::wildcard(MessageType::Event),
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
