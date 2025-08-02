// Copyright 2025 Cowboy AI, LLC.

//! NATS-based projection update mechanism
//!
//! This module provides a framework for building and updating read models (projections)
//! from the event stream using NATS JetStream.

use async_nats::jetstream::consumer::Consumer;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{
    error::{NatsError, Result},
    event_store::EventStore,
};
use crate::events::{EventEnvelope, GitDomainEvent};

/// Trait for projections that process events to build read models
#[async_trait]
pub trait Projection: Send + Sync {
    /// The name of this projection
    fn name(&self) -> &str;

    /// Get the current position (last processed sequence)
    async fn position(&self) -> Option<u64>;

    /// Save the current position
    async fn save_position(&self, sequence: u64) -> Result<()>;

    /// Apply an event to update the projection
    async fn apply(&mut self, envelope: &EventEnvelope, sequence: u64) -> Result<()>;

    /// Reset the projection to initial state
    async fn reset(&mut self) -> Result<()>;

    /// Check if this projection handles a specific event type
    fn handles_event_type(&self, event_type: &str) -> bool;
}

/// Projection manager that coordinates multiple projections
pub struct ProjectionManager {
    /// Event store
    event_store: Arc<EventStore>,

    /// Registered projections
    projections: Arc<RwLock<HashMap<String, Box<dyn Projection>>>>,

    /// Consumer group name
    consumer_group: String,

    /// Running state for each projection
    running_states: Arc<RwLock<HashMap<String, bool>>>,
}

impl ProjectionManager {
    /// Create a new projection manager
    pub fn new(event_store: Arc<EventStore>, consumer_group: String) -> Self {
        Self {
            event_store,
            projections: Arc::new(RwLock::new(HashMap::new())),
            consumer_group,
            running_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a projection
    pub async fn register(&self, projection: Box<dyn Projection>) -> Result<()> {
        let name = projection.name().to_string();
        let mut projections = self.projections.write().await;
        projections.insert(name.clone(), projection);
        info!("Registered projection: {}", name);
        Ok(())
    }

    /// Start all projections
    pub async fn start_all(&self) -> Result<()> {
        let projections = self.projections.read().await;

        for (name, _) in projections.iter() {
            self.start_projection(name).await?;
        }

        Ok(())
    }

    /// Start a specific projection
    pub async fn start_projection(&self, name: &str) -> Result<()> {
        let consumer_name = format!("{}_{}", self.consumer_group, name);

        // Create durable consumer for this projection
        let consumer = self
            .event_store
            .create_durable_consumer(&consumer_name, None)
            .await?;

        // Mark as running
        {
            let mut states = self.running_states.write().await;
            states.insert(name.to_string(), true);
        }

        // Clone for the spawned task
        let projections = self.projections.clone();
        let projection_name = name.to_string();
        let running_states = self.running_states.clone();

        // Spawn task to process events
        tokio::spawn(async move {
            let result = Self::process_events(consumer, projections, projection_name.clone()).await;

            // Mark as stopped when done
            let mut states = running_states.write().await;
            states.insert(projection_name.clone(), false);

            if let Err(e) = result {
                error!("Projection {} processing error: {}", projection_name, e);
            }
        });

        info!("Started projection: {}", name);
        Ok(())
    }

    /// Process events for a projection
    async fn process_events(
        consumer: Consumer<async_nats::jetstream::consumer::pull::Config>,
        projections: Arc<RwLock<HashMap<String, Box<dyn Projection>>>>,
        projection_name: String,
    ) -> Result<()> {
        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| NatsError::Other(format!("Failed to get messages: {}", e)))?;

        while let Some(Ok(message)) = messages.next().await {
            let sequence = message
                .info()
                .map_err(|e| NatsError::Other(format!("Failed to get message info: {}", e)))?
                .stream_sequence;

            // Parse the event envelope
            match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                Ok(envelope) => {
                    let event_type = envelope.event_type();

                    // Apply to projection
                    let mut projections = projections.write().await;
                    if let Some(projection) = projections.get_mut(&projection_name) {
                        if projection.handles_event_type(event_type) {
                            match projection.apply(&envelope, sequence).await {
                                Ok(_) => {
                                    debug!(
                                        "Projection {} processed event {} at seq {}",
                                        projection_name, event_type, sequence
                                    );

                                    // Save position
                                    if let Err(e) = projection.save_position(sequence).await {
                                        error!("Failed to save position: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Projection {} failed to process event: {}",
                                        projection_name, e
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse event envelope: {}", e);
                }
            }

            // Acknowledge the message
            if let Err(e) = message.ack().await {
                error!("Failed to acknowledge message: {}", e);
            }
        }

        Ok(())
    }

    /// Rebuild a projection from the beginning
    pub async fn rebuild_projection(&self, name: &str) -> Result<()> {
        let mut projections = self.projections.write().await;

        if let Some(projection) = projections.get_mut(name) {
            info!("Rebuilding projection: {}", name);

            // Reset the projection
            projection.reset().await?;

            // Get all events from the beginning
            let events = self.event_store.get_events_after(0, None).await?;

            // Apply each event
            for (idx, envelope) in events.iter().enumerate() {
                let sequence = (idx + 1) as u64;

                if projection.handles_event_type(envelope.event_type()) {
                    projection.apply(envelope, sequence).await?;
                }
            }

            // Save final position
            if !events.is_empty() {
                projection.save_position(events.len() as u64).await?;
            }

            info!("Rebuilt projection {} with {} events", name, events.len());
            Ok(())
        } else {
            Err(NatsError::Other(format!("Projection not found: {}", name)))
        }
    }

    /// Get projection status
    pub async fn status(&self) -> HashMap<String, ProjectionStatus> {
        let projections = self.projections.read().await;
        let mut status = HashMap::new();

        for (name, projection) in projections.iter() {
            let position = projection.position().await;
            status.insert(
                name.clone(),
                ProjectionStatus {
                    name: name.clone(),
                    position,
                    is_running: self
                        .running_states
                        .read()
                        .await
                        .get(name)
                        .copied()
                        .unwrap_or(false),
                },
            );
        }

        status
    }
}

/// Projection status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionStatus {
    /// Projection name
    pub name: String,

    /// Current position (last processed sequence)
    pub position: Option<u64>,

    /// Whether the projection is currently running
    pub is_running: bool,
}

/// Example: Repository statistics projection
pub struct RepositoryStatsProjection {
    /// Projection name
    name: String,

    /// Current position
    position: Arc<RwLock<Option<u64>>>,

    /// Repository statistics
    stats: Arc<RwLock<HashMap<String, RepositoryStats>>>,
}

/// Statistics about a git repository
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// The unique ID of the repository
    pub repository_id: String,
    /// Total number of commits
    pub commit_count: usize,
    /// Total number of branches
    pub branch_count: usize,
    /// Total number of tags
    pub tag_count: usize,
    /// Timestamp of the most recent commit
    pub last_commit_time: Option<DateTime<Utc>>,
    /// Total number of files analyzed
    pub total_files_analyzed: usize,
}

impl RepositoryStatsProjection {
    /// Create a new repository stats projection
    pub fn new(name: String) -> Self {
        Self {
            name,
            position: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get stats for a repository
    pub async fn get_stats(&self, repo_id: &str) -> Option<RepositoryStats> {
        let stats = self.stats.read().await;
        stats.get(repo_id).cloned()
    }

    /// Get all stats
    pub async fn get_all_stats(&self) -> HashMap<String, RepositoryStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }
}

#[async_trait]
impl Projection for RepositoryStatsProjection {
    fn name(&self) -> &str {
        &self.name
    }

    async fn position(&self) -> Option<u64> {
        let pos = self.position.read().await;
        *pos
    }

    async fn save_position(&self, sequence: u64) -> Result<()> {
        let mut pos = self.position.write().await;
        *pos = Some(sequence);
        Ok(())
    }

    async fn apply(&mut self, envelope: &EventEnvelope, _sequence: u64) -> Result<()> {
        let mut stats = self.stats.write().await;
        let repo_id = envelope.aggregate_id();

        let repo_stats = stats
            .entry(repo_id.clone())
            .or_insert_with(|| RepositoryStats {
                repository_id: repo_id.clone(),
                ..Default::default()
            });

        match &envelope.event {
            GitDomainEvent::CommitAnalyzed(e) => {
                repo_stats.commit_count += 1;
                repo_stats.last_commit_time = Some(e.commit_timestamp);
            }
            GitDomainEvent::BranchCreated(_) => {
                repo_stats.branch_count += 1;
            }
            GitDomainEvent::BranchDeleted(_) => {
                repo_stats.branch_count = repo_stats.branch_count.saturating_sub(1);
            }
            GitDomainEvent::TagCreated(_) => {
                repo_stats.tag_count += 1;
            }
            GitDomainEvent::FileAnalyzed(_) => {
                repo_stats.total_files_analyzed += 1;
            }
            _ => {} // Ignore other events
        }

        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        let mut stats = self.stats.write().await;
        stats.clear();

        let mut pos = self.position.write().await;
        *pos = None;

        Ok(())
    }

    fn handles_event_type(&self, event_type: &str) -> bool {
        matches!(
            event_type,
            "CommitAnalyzed" | "BranchCreated" | "BranchDeleted" | "TagCreated" | "FileAnalyzed"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::RepositoryId;
    use crate::events::{CommitAnalyzed, EventEnvelope};
    use crate::value_objects::{AuthorInfo, CommitHash};

    #[tokio::test]
    async fn test_repository_stats_projection() {
        let projection = RepositoryStatsProjection::new("test_stats".to_string());
        let repo_id = RepositoryId::new();

        // Create test event
        let event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
            repository_id: repo_id,
            commit_hash: CommitHash::new("abc123d").unwrap(),
            parents: vec![],
            author: AuthorInfo {
                name: "Test Author".to_string(),
                email: "test@example.com".to_string(),
            },
            message: "Test commit".to_string(),
            files_changed: vec![],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event);

        // Apply event
        let mut proj = projection;
        proj.apply(&envelope, 1).await.unwrap();

        // Check stats
        let stats = proj.get_stats(&repo_id.to_string()).await.unwrap();
        assert_eq!(stats.commit_count, 1);
        assert!(stats.last_commit_time.is_some());
    }
}
