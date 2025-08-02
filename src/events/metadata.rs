// Copyright 2025 Cowboy AI, LLC.

//! Event metadata for correlation and causation tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event metadata that tracks correlation and causation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: Uuid,

    /// Correlation ID - tracks related events across a business transaction
    pub correlation_id: Uuid,

    /// Causation ID - the ID of the event that caused this event
    pub causation_id: Uuid,

    /// User ID who initiated the action (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Timestamp when the event occurred
    pub occurred_at: DateTime<Utc>,

    /// Version of the event schema
    pub schema_version: u32,
}

impl EventMetadata {
    /// Create new metadata with a new correlation chain
    pub fn new() -> Self {
        let event_id = Uuid::new_v4();
        Self {
            event_id,
            correlation_id: event_id, // Start new correlation chain
            causation_id: event_id,   // Self-caused (root event)
            user_id: None,
            occurred_at: Utc::now(),
            schema_version: 1,
        }
    }

    /// Create metadata for a follow-up event in the same correlation chain
    pub fn from_correlation(correlation_id: Uuid, causation_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_id,
            causation_id,
            user_id: None,
            occurred_at: Utc::now(),
            schema_version: 1,
        }
    }

    /// Create metadata from a command
    pub fn from_command(command_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_id: command_id, // Command starts the correlation
            causation_id: command_id,   // Command caused the event
            user_id: None,
            occurred_at: Utc::now(),
            schema_version: 1,
        }
    }

    /// Set the user ID
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for events with metadata
pub trait WithMetadata {
    /// Get the event metadata
    fn metadata(&self) -> &EventMetadata;

    /// Get mutable event metadata
    fn metadata_mut(&mut self) -> &mut EventMetadata;

    /// Get the event ID
    fn event_id(&self) -> Uuid {
        self.metadata().event_id
    }

    /// Get the correlation ID
    fn correlation_id(&self) -> Uuid {
        self.metadata().correlation_id
    }

    /// Get the causation ID
    fn causation_id(&self) -> Uuid {
        self.metadata().causation_id
    }

    /// Get when the event occurred
    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata().occurred_at
    }
}

/// Context for tracking correlation across operations
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Current correlation ID
    pub correlation_id: Uuid,

    /// Stack of causation IDs
    causation_stack: Vec<Uuid>,

    /// User ID if known
    pub user_id: Option<String>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        Self {
            correlation_id: id,
            causation_stack: vec![id],
            user_id: None,
        }
    }

    /// Create context from existing correlation
    pub fn from_correlation(correlation_id: Uuid, causation_id: Uuid) -> Self {
        Self {
            correlation_id,
            causation_stack: vec![causation_id],
            user_id: None,
        }
    }

    /// Get the current causation ID
    pub fn causation_id(&self) -> Uuid {
        *self.causation_stack.last().unwrap_or(&self.correlation_id)
    }

    /// Push a new causation ID (when handling an event)
    pub fn push_causation(&mut self, event_id: Uuid) {
        self.causation_stack.push(event_id);
    }

    /// Pop the last causation ID
    pub fn pop_causation(&mut self) {
        if self.causation_stack.len() > 1 {
            self.causation_stack.pop();
        }
    }

    /// Create event metadata from this context
    pub fn create_metadata(&self) -> EventMetadata {
        let mut metadata =
            EventMetadata::from_correlation(self.correlation_id, self.causation_id());

        if let Some(ref user_id) = self.user_id {
            metadata.user_id = Some(user_id.clone());
        }

        metadata
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new();
        assert_eq!(metadata.event_id, metadata.correlation_id);
        assert_eq!(metadata.event_id, metadata.causation_id);
    }

    #[test]
    fn test_correlation_chain() {
        let root = EventMetadata::new();
        let child = EventMetadata::from_correlation(root.correlation_id, root.event_id);

        assert_eq!(child.correlation_id, root.correlation_id);
        assert_eq!(child.causation_id, root.event_id);
        assert_ne!(child.event_id, root.event_id);
    }

    #[test]
    fn test_correlation_context() {
        let mut context = CorrelationContext::new();
        let initial_causation = context.causation_id();

        // Push new causation
        let event_id = Uuid::new_v4();
        context.push_causation(event_id);
        assert_eq!(context.causation_id(), event_id);

        // Pop causation
        context.pop_causation();
        assert_eq!(context.causation_id(), initial_causation);
    }
}
