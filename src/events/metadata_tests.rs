// Copyright 2025 Cowboy AI, LLC.

//! Tests for event metadata functionality

#[cfg(test)]
mod tests {
    use super::super::metadata::*;
    use uuid::Uuid;

    #[test]
    fn test_event_metadata_new() {
        let metadata = EventMetadata::new();
        
        // Verify defaults
        assert_eq!(metadata.correlation_id, metadata.event_id);
        assert_eq!(metadata.causation_id, metadata.event_id);
        assert_eq!(metadata.schema_version, 1);
        assert!(metadata.user_id.is_none());
    }

    #[test]
    fn test_event_metadata_from_command() {
        let command_id = Uuid::new_v4();
        let metadata = EventMetadata::from_command(command_id);
        
        assert_eq!(metadata.correlation_id, command_id);
        assert_eq!(metadata.causation_id, command_id);
        assert_ne!(metadata.event_id, command_id); // Event ID should be unique
    }

    #[test]
    fn test_event_metadata_from_correlation() {
        let correlation_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();
        let metadata = EventMetadata::from_correlation(correlation_id, causation_id);
        
        assert_eq!(metadata.correlation_id, correlation_id);
        assert_eq!(metadata.causation_id, causation_id);
        assert_ne!(metadata.event_id, correlation_id);
        assert_ne!(metadata.event_id, causation_id);
    }

    #[test]
    fn test_event_metadata_with_user() {
        let metadata = EventMetadata::new().with_user("test-user".to_string());
        
        assert_eq!(metadata.user_id, Some("test-user".to_string()));
    }


    #[test]
    fn test_event_metadata_builder_chain() {
        let metadata = EventMetadata::new()
            .with_user("admin".to_string());
        
        assert_eq!(metadata.user_id, Some("admin".to_string()));
    }

    #[test]
    fn test_correlation_context_new() {
        let context = CorrelationContext::new();
        
        // Should create new correlation ID
        assert_ne!(context.correlation_id, Uuid::nil());
        assert_eq!(context.causation_id(), context.correlation_id);
        assert!(context.user_id.is_none());
    }

    #[test]
    fn test_correlation_context_from_correlation() {
        let parent = CorrelationContext::new();
        let child = CorrelationContext::from_correlation(
            parent.correlation_id,
            parent.causation_id()
        );
        
        assert_eq!(child.correlation_id, parent.correlation_id);
        assert_eq!(child.causation_id(), parent.causation_id());
    }


    #[test]
    fn test_with_metadata_trait() {
        struct TestEvent {
            data: String,
            metadata: EventMetadata,
        }

        impl WithMetadata for TestEvent {
            fn metadata(&self) -> &EventMetadata {
                &self.metadata
            }

            fn metadata_mut(&mut self) -> &mut EventMetadata {
                &mut self.metadata
            }
        }

        let event = TestEvent {
            data: "test".to_string(),
            metadata: EventMetadata::new().with_user("user1".to_string()),
        };
        
        assert_eq!(event.data, "test");
        assert_eq!(event.metadata().user_id, Some("user1".to_string()));
        assert_eq!(event.event_id(), event.metadata().event_id);
        assert_eq!(event.correlation_id(), event.metadata().correlation_id);
    }

    #[test]
    fn test_event_metadata_serialization() {
        let metadata = EventMetadata::new()
            .with_user("test-user".to_string());
        
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EventMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.event_id, metadata.event_id);
        assert_eq!(deserialized.user_id, metadata.user_id);
        assert_eq!(deserialized.schema_version, metadata.schema_version);
    }


    #[test]
    fn test_metadata_timestamp_ordering() {
        let metadata1 = EventMetadata::new();
        
        // Sleep briefly to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let metadata2 = EventMetadata::new();
        
        assert!(metadata2.occurred_at > metadata1.occurred_at);
    }
}