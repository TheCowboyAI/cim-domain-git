// Copyright 2025 Cowboy AI, LLC.

//! Example demonstrating distributed tracing with NATS
//!
//! This example shows how to:
//! - Initialize OpenTelemetry tracing
//! - Trace commands through the system
//! - Trace events and their propagation
//! - View trace context in projections

use async_nats::HeaderMap;
use chrono::Utc;
use cim_domain_git::{
    aggregate::{Repository, RepositoryId},
    commands::CloneRepository,
    events::{EventEnvelope, GitDomainEvent, RepositoryCloned},
    nats::{
        EventPublisher, EventStore, EventStoreConfig, NatsClient, NatsConfig, TracingConfig,
        TracingManager,
    },
    value_objects::RemoteUrl,
};
use opentelemetry::{
    global,
    trace::{Span, TraceContextExt, Tracer},
    Context, KeyValue,
};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize console logging
    tracing_subscriber::fmt::init();

    info!("=== Distributed Tracing Demo ===");

    // Step 1: Initialize distributed tracing
    info!("\n=== Initializing Distributed Tracing ===");

    let tracing_config = TracingConfig {
        service_name: "git-domain-demo".to_string(),
        otlp_endpoint: Some("http://localhost:4317".to_string()), // Jaeger or OTLP collector
        sampling_ratio: 1.0,                                      // Sample all traces for demo
        console_debug: true,                                      // Also print to console
        resource_attributes: {
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("deployment.environment".to_string(), "demo".to_string());
            attrs.insert("service.namespace".to_string(), "cim".to_string());
            attrs
        },
    };

    let tracing_manager = Arc::new(TracingManager::init(tracing_config)?);
    info!("Initialized OpenTelemetry tracing");

    // Step 2: Connect to NATS with tracing
    info!("\n=== Connecting to NATS ===");

    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };

    let client = NatsClient::connect(config).await?;
    info!("Connected to NATS with JetStream");

    // Create event publisher with tracing
    let event_publisher = Arc::new(EventPublisher::with_tracing(
        client.client.clone(),
        "git".to_string(),
        tracing_manager.clone(),
    ));

    let event_store = Arc::new(
        EventStore::new(
            client.jetstream.clone(),
            event_publisher.clone(),
            EventStoreConfig::default(),
        )
        .await?,
    );

    // Step 3: Demonstrate command tracing
    info!("\n=== Tracing a Command ===");

    let command_id = Uuid::new_v4();
    let command = CloneRepository {
        remote_url: RemoteUrl::new("https://github.com/example/traced-repo.git")?,
        local_path: "/tmp/traced-repo".to_string(),
        branch: Some("main".to_string()),
    };

    // Create a root span for the command
    let command_span = tracing_manager.command_span("CloneRepository", command_id);
    let command_context = Context::current_with_span(command_span.clone());

    // Add command details to span
    command_span.set_attributes(vec![
        KeyValue::new("repository.url", command.remote_url.to_string()),
        KeyValue::new(
            "repository.branch",
            command.branch.as_deref().unwrap_or("default"),
        ),
    ]);

    // Create headers with trace context
    let mut headers = HeaderMap::new();
    headers.insert("X-Command-ID", command_id.to_string());
    headers.insert("X-Command-Type", "CloneRepository");
    tracing_manager.inject_context(&command_context, &mut headers);

    // Log trace information
    if let Some(trace_ctx) = headers.get("traceparent") {
        info!("Trace context: {}", trace_ctx.as_str());
    }

    // Simulate command processing with child spans
    {
        let _guard = command_context.attach();

        // Child span for validation
        let validation_span = global::tracer("git-domain-demo")
            .span_builder("validate_repository_url")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start(&global::tracer("git-domain-demo"));

        validation_span.add_event("url_validated", vec![KeyValue::new("url.valid", true)]);
        validation_span.end();

        // Child span for cloning
        let clone_span = global::tracer("git-domain-demo")
            .span_builder("git_clone")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(vec![
                KeyValue::new("git.operation", "clone"),
                KeyValue::new("git.remote", command.remote_url.to_string()),
            ])
            .start(&global::tracer("git-domain-demo"));

        // Simulate clone operation
        sleep(Duration::from_millis(100)).await;

        clone_span.add_event(
            "clone_completed",
            vec![
                KeyValue::new("repository.size_bytes", 1024 * 1024),
                KeyValue::new("repository.file_count", 42),
            ],
        );
        clone_span.end();
    }

    // Step 4: Trace event publishing
    info!("\n=== Tracing Event Publishing ===");

    let repo_id = RepositoryId::new();
    let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: command.remote_url.clone(),
        local_path: command.local_path.clone(),
        timestamp: Utc::now(),
    });

    // Create event with correlation to the command
    let envelope = EventEnvelope::from_command(event, command_id);

    // Publish event (tracing is handled internally by the publisher)
    event_store.append(&envelope).await?;

    info!("Published traced event with correlation to command");

    // Step 5: Demonstrate trace propagation through multiple events
    info!("\n=== Trace Propagation Chain ===");

    // Create a chain of correlated events
    let mut previous_event_id = envelope.event_id();

    for i in 1..=3 {
        let event = GitDomainEvent::CommitAnalyzed(cim_domain_git::events::CommitAnalyzed {
            repository_id: repo_id,
            commit_hash: cim_domain_git::value_objects::CommitHash::new(&format!("abc{}", i))?,
            parents: vec![],
            author: cim_domain_git::value_objects::AuthorInfo {
                name: format!("Traced Author {}", i),
                email: "traced@example.com".to_string(),
            },
            message: format!("Traced commit {}", i),
            files_changed: vec![],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });

        // Create event correlated to the previous one
        let envelope = EventEnvelope::from_correlation(
            event,
            envelope.correlation_id(), // Keep same correlation
            previous_event_id,         // Previous event is the cause
        );

        event_store.append(&envelope).await?;
        previous_event_id = envelope.event_id();

        info!("Published event {} in trace chain", i);
        sleep(Duration::from_millis(50)).await;
    }

    // Step 6: Query with tracing
    info!("\n=== Tracing Query Operations ===");

    let query_span = tracing_manager.query_span("GetRepositoryEvents");
    let query_context = Context::current_with_span(query_span.clone());

    {
        let _guard = query_context.attach();

        // Trace the query execution
        let events = event_store.load_aggregate_events(&repo_id).await?;

        query_span.add_event(
            "query_completed",
            vec![
                KeyValue::new("event_count", events.len() as i64),
                KeyValue::new("repository_id", repo_id.to_string()),
            ],
        );

        info!("Loaded {} events with tracing", events.len());
    }

    query_span.end();
    command_span.end();

    // Step 7: Demonstrate error tracing
    info!("\n=== Error Tracing ===");

    let error_span = tracing_manager.command_span("FailingCommand", Uuid::new_v4());

    // Simulate an error
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "Repository not found");
    tracing_manager.record_error(&error_span, &error);

    error_span.end();

    // Give time for traces to be exported
    info!("\n=== Waiting for trace export ===");
    sleep(Duration::from_secs(2)).await;

    // Shutdown tracing
    global::shutdown_tracer_provider();

    info!("\n=== Tracing Demo Complete ===");
    info!("Distributed tracing features demonstrated:");
    info!("✓ Command tracing with child spans");
    info!("✓ Event publishing with trace context");
    info!("✓ Correlation and causation tracking");
    info!("✓ Query operation tracing");
    info!("✓ Error recording in traces");
    info!("\nView traces in Jaeger UI at http://localhost:16686");

    Ok(())
}
