// Copyright 2025 Cowboy AI, LLC.

//! Event publisher for the Git domain

use async_nats::{Client, HeaderMap};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, info};
use uuid::Uuid;

use super::{
    error::{NatsError, Result},
    subject::{EventAction, GitSubject, SubjectMapper},
};
use crate::aggregate::RepositoryId;
use crate::events::{EventEnvelope, GitDomainEvent};

/// Trait for Git domain events that can be published
pub trait EventPublishing: Serialize {
    /// Get the event ID
    fn event_id(&self) -> Uuid;

    /// Get the event type name
    fn event_type(&self) -> &'static str;

    /// Get the correlation ID
    fn correlation_id(&self) -> Uuid;

    /// Get the causation ID
    fn causation_id(&self) -> Uuid;

    /// Get the aggregate ID
    fn aggregate_id(&self) -> String;

    /// Get when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// Event publisher for Git domain events
pub struct EventPublisher {
    /// NATS client
    client: Client,

    /// Subject prefix (usually "git")
    subject_prefix: String,
}

impl EventPublisher {
    /// Create a new event publisher
    pub fn new(client: Client, subject_prefix: String) -> Self {
        Self {
            client,
            subject_prefix,
        }
    }

    /// Publish a domain event
    pub async fn publish_event(&self, event: &GitDomainEvent) -> Result<()> {
        // Get event metadata based on the event type
        let (event_type, event_id, aggregate_id, timestamp) = match event {
            GitDomainEvent::RepositoryCloned(e) => (
                "RepositoryCloned",
                Uuid::new_v4(), // Event ID generated here since events don't have built-in IDs
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::CommitAnalyzed(e) => (
                "CommitAnalyzed",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::BranchCreated(e) => (
                "BranchCreated",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::BranchDeleted(e) => (
                "BranchDeleted",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::TagCreated(e) => (
                "TagCreated",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::RepositoryMetadataUpdated(e) => (
                "RepositoryMetadataUpdated",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::MergeDetected(e) => (
                "MergeDetected",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::FileAnalyzed(e) => (
                "FileAnalyzed",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
            GitDomainEvent::RepositoryAnalyzed(e) => (
                "RepositoryAnalyzed",
                Uuid::new_v4(),
                e.repository_id.to_string(),
                e.timestamp,
            ),
        };

        // Map to NATS subject
        let subject = SubjectMapper::event_subject(event_type).ok_or_else(|| {
            NatsError::InvalidSubject(format!("Unknown event type: {}", event_type))
        })?;

        let subject_str = subject.to_string();
        debug!("Publishing event {} to subject {}", event_type, subject_str);

        // Create headers with event metadata
        let mut headers = HeaderMap::new();
        headers.insert("X-Event-ID", event_id.to_string());
        headers.insert("X-Event-Type", event_type.to_string());
        headers.insert("X-Correlation-ID", Uuid::new_v4().to_string()); // Use EventEnvelope for proper correlation tracking
        headers.insert("X-Causation-ID", event_id.to_string()); // Use EventEnvelope for proper causation tracking
        headers.insert("X-Aggregate-ID", aggregate_id);
        headers.insert("X-Timestamp", timestamp.to_rfc3339());
        headers.insert("X-Domain", self.subject_prefix.clone());

        // Serialize the event
        let payload =
            serde_json::to_vec(&event).map_err(|e| NatsError::SerializationError(e.to_string()))?;

        // Publish with headers
        self.client
            .publish_with_headers(subject_str, headers, Bytes::from(payload))
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        info!("Published event {} with ID {}", event_type, event_id);

        Ok(())
    }

    /// Publish multiple events
    pub async fn publish_events(&self, events: &[GitDomainEvent]) -> Result<()> {
        for event in events {
            self.publish_event(event).await?;
        }
        Ok(())
    }

    /// Publish an event envelope with full metadata
    pub async fn publish_envelope(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_type = envelope.event_type();

        // Map to NATS subject
        let subject = SubjectMapper::event_subject(event_type).ok_or_else(|| {
            NatsError::InvalidSubject(format!("Unknown event type: {}", event_type))
        })?;

        let subject_str = subject.to_string();
        debug!("Publishing event {} to subject {}", event_type, subject_str);

        // Create headers with event metadata
        let mut headers = HeaderMap::new();
        headers.insert("X-Event-ID", envelope.event_id().to_string());
        headers.insert("X-Event-Type", event_type.to_string());
        headers.insert("X-Correlation-ID", envelope.correlation_id().to_string());
        headers.insert("X-Causation-ID", envelope.causation_id().to_string());
        headers.insert("X-Aggregate-ID", envelope.aggregate_id());
        headers.insert("X-Timestamp", envelope.occurred_at().to_rfc3339());
        headers.insert("X-Domain", self.subject_prefix.clone());
        headers.insert(
            "X-Schema-Version",
            envelope.metadata.schema_version.to_string(),
        );

        if let Some(ref user_id) = envelope.metadata.user_id {
            headers.insert("X-User-ID", user_id.clone());
        }

        // Serialize the entire envelope
        let payload = serde_json::to_vec(&envelope)
            .map_err(|e| NatsError::SerializationError(e.to_string()))?;

        // Publish with headers
        let result = self
            .client
            .publish_with_headers(subject_str.clone(), headers, Bytes::from(payload))
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()));

        if result.is_ok() {
            info!(
                "Published event {} with ID {} (correlation: {}, causation: {})",
                event_type,
                envelope.event_id(),
                envelope.correlation_id(),
                envelope.causation_id()
            );
        }

        result
    }

    /// Publish multiple event envelopes
    pub async fn publish_envelopes(&self, envelopes: &[EventEnvelope]) -> Result<()> {
        for envelope in envelopes {
            self.publish_envelope(envelope).await?;
        }
        Ok(())
    }

    /// Publish a raw event (for special cases)
    pub async fn publish_raw(
        &self,
        action: EventAction,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<()> {
        let subject = GitSubject::event(action);
        let subject_str = subject.to_string();

        self.client
            .publish_with_headers(subject_str, headers, payload)
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        Ok(())
    }

    /// Create a simple event publisher for basic events
    pub async fn publish_simple(
        &self,
        event_type: &str,
        repository_id: &RepositoryId,
        data: serde_json::Value,
    ) -> Result<()> {
        let subject = SubjectMapper::event_subject(event_type).ok_or_else(|| {
            NatsError::InvalidSubject(format!("Unknown event type: {}", event_type))
        })?;

        let subject_str = subject.to_string();

        // Create event wrapper
        let event = serde_json::json!({
            "event_type": event_type,
            "event_id": Uuid::new_v4().to_string(),
            "repository_id": repository_id.to_string(),
            "timestamp": Utc::now().to_rfc3339(),
            "data": data,
        });

        let payload =
            serde_json::to_vec(&event).map_err(|e| NatsError::SerializationError(e.to_string()))?;

        self.client
            .publish(subject_str, Bytes::from(payload))
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::RepositoryCloned;
    use crate::value_objects::RemoteUrl;

    #[test]
    fn test_event_subject_mapping() {
        let subject = SubjectMapper::event_subject("RepositoryCloned");
        assert!(subject.is_some());
        assert_eq!(subject.unwrap().to_string(), "git.event.repository.cloned");
    }

    #[test]
    fn test_unknown_event_type() {
        let subject = SubjectMapper::event_subject("UnknownEvent");
        assert!(subject.is_none());
    }
}
