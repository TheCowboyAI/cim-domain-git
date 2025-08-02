// Copyright 2025 Cowboy AI, LLC.

//! NATS integration example for Git domain
//!
//! This example demonstrates:
//! - Connecting to NATS
//! - Publishing Git domain events
//! - Subscribing to commands and events
//! - Health checks and service discovery

use chrono::Utc;
use cim_domain_git::{
    aggregate::RepositoryId,
    events::{CommitAnalyzed, FileChangeInfo, FileChangeType, GitDomainEvent, RepositoryCloned},
    nats::{
        health::NatsHealthCheck, CommandHandler, CommandSubscriber, EventHandler, EventSubscriber,
        HealthService, ServiceDiscovery, ServiceInfo, ServiceStatus,
    },
    value_objects::{AuthorInfo, CommitHash, FilePath, RemoteUrl},
    EventPublisher, NatsClient, NatsConfig,
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Git domain NATS integration demo");

    // Configure NATS
    let config = NatsConfig::from_env().unwrap_or_default();

    // Connect to NATS
    let client = NatsClient::connect(config.clone()).await?;
    info!("Connected to NATS");

    // Create event publisher
    let publisher = EventPublisher::new(client.client().clone(), "git".to_string());

    // Publish some example events
    publish_example_events(&publisher).await?;

    // Set up command subscriber
    let command_subscriber = CommandSubscriber::new(client.client().clone());

    // Register a command handler
    command_subscriber
        .register_handler(CloneRepositoryHandler)
        .await;

    // Start command subscriber in background
    let cmd_handle = tokio::spawn(async move {
        if let Err(e) = command_subscriber.start().await {
            eprintln!("Command subscriber error: {}", e);
        }
    });

    // Set up event subscriber
    let event_subscriber = EventSubscriber::new(client.client().clone());

    // Register event handlers
    event_subscriber
        .register_handler(RepositoryClonedHandler)
        .await;
    event_subscriber
        .register_handler(CommitAnalyzedHandler)
        .await;

    // Start event subscriber in background
    let event_handle = tokio::spawn(async move {
        if let Err(e) = event_subscriber.start().await {
            eprintln!("Event subscriber error: {}", e);
        }
    });

    // Set up health service
    let service_info = ServiceInfo {
        id: Uuid::new_v4().to_string(),
        name: "git-domain-example".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: Some("Git domain NATS example".to_string()),
        endpoints: vec!["git.>".to_string()],
        metadata: HashMap::new(),
        last_heartbeat: Utc::now(),
        status: ServiceStatus::Healthy,
    };

    let health_service = HealthService::new(client.client().clone(), service_info);

    // Register NATS connection health check
    health_service
        .register_check(
            "nats".to_string(),
            Box::new(NatsHealthCheck::new(client.client().clone())),
        )
        .await;

    // Start health service in background
    let health_handle = tokio::spawn(async move {
        if let Err(e) = health_service.start().await {
            eprintln!("Health service error: {}", e);
        }
    });

    // Demonstrate service discovery
    let discovery = ServiceDiscovery::new(client.client().clone());

    info!("Waiting for services to register...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Try to discover git domain services
    match discovery.discover("git-domain-example").await {
        Ok(services) => {
            info!("Discovered {} services", services.len());
            for service in services {
                info!("  - {} ({})", service.name, service.id);
            }
        }
        Err(e) => {
            info!(
                "Service discovery failed (expected if no other instances): {}",
                e
            );
        }
    }

    // Check health
    match discovery.check_health("git-domain-example").await {
        Ok(health) => {
            info!("Health check result: {:?}", health.status);
        }
        Err(e) => {
            info!(
                "Health check failed (expected if no other instances): {}",
                e
            );
        }
    }

    info!("Demo running... Press Ctrl+C to stop");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    // Clean up
    client.drain().await?;

    Ok(())
}

async fn publish_example_events(
    publisher: &EventPublisher,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Publishing example events");

    // Create a repository cloned event
    let repo_id = RepositoryId::new();
    let cloned_event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/example/repo.git")?,
        local_path: "/tmp/example-repo".to_string(),
        timestamp: Utc::now(),
    });

    publisher.publish_event(&cloned_event).await?;
    info!("Published RepositoryCloned event");

    // Create a commit analyzed event
    let analyzed_event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
        repository_id: repo_id,
        commit_hash: CommitHash::new("abc123def456789")?,
        parents: vec![],
        author: AuthorInfo {
            name: "Example Author".to_string(),
            email: "author@example.com".to_string(),
        },
        message: "Initial commit".to_string(),
        files_changed: vec![FileChangeInfo {
            path: FilePath::new("README.md")?,
            change_type: FileChangeType::Added,
            additions: 10,
            deletions: 0,
        }],
        commit_timestamp: Utc::now(),
        timestamp: Utc::now(),
    });

    publisher.publish_event(&analyzed_event).await?;
    info!("Published CommitAnalyzed event");

    Ok(())
}

// Example command handler
struct CloneRepositoryHandler;

#[async_trait::async_trait]
impl CommandHandler for CloneRepositoryHandler {
    type Command = serde_json::Value;
    type Result = serde_json::Value;

    async fn handle(&self, command: Self::Command) -> cim_domain_git::nats::Result<Self::Result> {
        info!("Handling CloneRepository command: {:?}", command);

        // Return acknowledgment
        Ok(serde_json::json!({
            "status": "accepted",
            "message": "Repository clone initiated"
        }))
    }

    fn command_type(&self) -> &'static str {
        "CloneRepository"
    }
}

// Example event handlers
struct RepositoryClonedHandler;

#[async_trait::async_trait]
impl EventHandler for RepositoryClonedHandler {
    type Event = serde_json::Value;

    async fn handle(&self, event: Self::Event) -> cim_domain_git::nats::Result<()> {
        info!("Handling RepositoryCloned event: {:?}", event);
        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "RepositoryCloned"
    }
}

struct CommitAnalyzedHandler;

#[async_trait::async_trait]
impl EventHandler for CommitAnalyzedHandler {
    type Event = serde_json::Value;

    async fn handle(&self, event: Self::Event) -> cim_domain_git::nats::Result<()> {
        info!("Handling CommitAnalyzed event: {:?}", event);
        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "CommitAnalyzed"
    }
}
