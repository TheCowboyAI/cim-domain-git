// Copyright 2025 Cowboy AI, LLC.

//! Event envelope for wrapping domain events with metadata

use super::metadata::EventMetadata;
use super::GitDomainEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event envelope that wraps a domain event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Event metadata
    pub metadata: EventMetadata,

    /// The domain event
    pub event: GitDomainEvent,
}

impl EventEnvelope {
    /// Create a new event envelope
    pub fn new(event: GitDomainEvent) -> Self {
        Self {
            metadata: EventMetadata::new(),
            event,
        }
    }

    /// Create an envelope with specific metadata
    pub fn with_metadata(event: GitDomainEvent, metadata: EventMetadata) -> Self {
        Self { metadata, event }
    }

    /// Create an envelope from a command
    pub fn from_command(event: GitDomainEvent, command_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::from_command(command_id),
            event,
        }
    }

    /// Create an envelope in a correlation chain
    pub fn from_correlation(
        event: GitDomainEvent,
        correlation_id: Uuid,
        causation_id: Uuid,
    ) -> Self {
        Self {
            metadata: EventMetadata::from_correlation(correlation_id, causation_id),
            event,
        }
    }

    /// Get the event ID
    pub fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    /// Get the correlation ID
    pub fn correlation_id(&self) -> Uuid {
        self.metadata.correlation_id
    }

    /// Get the causation ID
    pub fn causation_id(&self) -> Uuid {
        self.metadata.causation_id
    }

    /// Get when the event occurred
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match &self.event {
            GitDomainEvent::RepositoryCloned(_) => "RepositoryCloned",
            GitDomainEvent::CommitAnalyzed(_) => "CommitAnalyzed",
            GitDomainEvent::BranchCreated(_) => "BranchCreated",
            GitDomainEvent::BranchDeleted(_) => "BranchDeleted",
            GitDomainEvent::TagCreated(_) => "TagCreated",
            GitDomainEvent::RepositoryMetadataUpdated(_) => "RepositoryMetadataUpdated",
            GitDomainEvent::MergeDetected(_) => "MergeDetected",
            GitDomainEvent::FileAnalyzed(_) => "FileAnalyzed",
            GitDomainEvent::RepositoryAnalyzed(_) => "RepositoryAnalyzed",
        }
    }

    /// Get the aggregate ID from the event
    pub fn aggregate_id(&self) -> String {
        match &self.event {
            GitDomainEvent::RepositoryCloned(e) => e.repository_id.to_string(),
            GitDomainEvent::CommitAnalyzed(e) => e.repository_id.to_string(),
            GitDomainEvent::BranchCreated(e) => e.repository_id.to_string(),
            GitDomainEvent::BranchDeleted(e) => e.repository_id.to_string(),
            GitDomainEvent::TagCreated(e) => e.repository_id.to_string(),
            GitDomainEvent::RepositoryMetadataUpdated(e) => e.repository_id.to_string(),
            GitDomainEvent::MergeDetected(e) => e.repository_id.to_string(),
            GitDomainEvent::FileAnalyzed(e) => e.repository_id.to_string(),
            GitDomainEvent::RepositoryAnalyzed(e) => e.repository_id.to_string(),
        }
    }
}

/// Builder for creating event envelopes
pub struct EventEnvelopeBuilder {
    metadata: EventMetadata,
}

impl EventEnvelopeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            metadata: EventMetadata::new(),
        }
    }

    /// Set the correlation ID
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.metadata.correlation_id = correlation_id;
        self
    }

    /// Set the causation ID
    pub fn with_causation(mut self, causation_id: Uuid) -> Self {
        self.metadata.causation_id = causation_id;
        self
    }

    /// Set the user ID
    pub fn with_user(mut self, user_id: String) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    /// Build the envelope with the given event
    pub fn build(self, event: GitDomainEvent) -> EventEnvelope {
        EventEnvelope {
            metadata: self.metadata,
            event,
        }
    }
}

impl Default for EventEnvelopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::RepositoryId;
    use crate::events::RepositoryCloned;
    use crate::value_objects::RemoteUrl;

    #[test]
    fn test_event_envelope_creation() {
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event.clone());
        assert_eq!(envelope.event_type(), "RepositoryCloned");
        assert_eq!(envelope.correlation_id(), envelope.event_id());
    }

    #[test]
    fn test_event_envelope_builder() {
        let correlation_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();

        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelopeBuilder::new()
            .with_correlation(correlation_id)
            .with_causation(causation_id)
            .with_user("user123".to_string())
            .build(event);

        assert_eq!(envelope.correlation_id(), correlation_id);
        assert_eq!(envelope.causation_id(), causation_id);
        assert_eq!(envelope.metadata.user_id, Some("user123".to_string()));
    }
}
