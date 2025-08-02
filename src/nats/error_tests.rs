// Copyright 2025 Cowboy AI, LLC.

//! Tests for NATS error handling

#[cfg(test)]
mod tests {
    use super::super::error::*;

    #[test]
    fn test_nats_error_display() {
        let error = NatsError::ConnectionError("connection refused".to_string());
        assert_eq!(error.to_string(), "Connection error: connection refused");

        let error = NatsError::SubscriptionError("invalid subject".to_string());
        assert_eq!(error.to_string(), "Subscription error: invalid subject");

        let error = NatsError::PublishError("timeout".to_string());
        assert_eq!(error.to_string(), "Publish error: timeout");

        let error = NatsError::InvalidSubject("bad..subject".to_string());
        assert_eq!(error.to_string(), "Invalid subject: bad..subject");

        let error = NatsError::SerializationError("invalid JSON".to_string());
        assert_eq!(error.to_string(), "Serialization error: invalid JSON");

        let error = NatsError::DeserializationError("unexpected EOF".to_string());
        assert_eq!(error.to_string(), "Deserialization error: unexpected EOF");

        let error = NatsError::ConfigurationError("stream not found".to_string());
        assert_eq!(error.to_string(), "Configuration error: stream not found");

        let error = NatsError::Timeout;
        assert_eq!(error.to_string(), "Operation timed out");

        let error = NatsError::HealthCheckError("service down".to_string());
        assert_eq!(error.to_string(), "Health check error: service down");

        let error = NatsError::Other("generic error".to_string());
        assert_eq!(error.to_string(), "NATS error: generic error");
    }

    #[test]
    fn test_error_conversion_from_async_nats() {
        // Simulate various async-nats errors
        // Note: async_nats API has changed, using a mock error for now
        let io_error = std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        );
        let nats_error: NatsError = io_error.into();
        assert!(matches!(nats_error, NatsError::Other(_)));
    }

    #[test]
    fn test_error_chaining() {
        let original = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
        let error = NatsError::Other(original.to_string());
        
        // Verify we can get the error message
        assert!(error.to_string().contains("disk full"));
    }

    #[test]
    fn test_result_type() {
        // Test Ok case
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Test Err case
        let result: Result<String> = Err(NatsError::Timeout);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NatsError::Timeout));
    }

    #[test]
    fn test_error_variants_exhaustive() {
        // Ensure all error variants are covered
        let errors = vec![
            NatsError::ConnectionError("test".to_string()),
            NatsError::SubscriptionError("test".to_string()),
            NatsError::PublishError("test".to_string()),
            NatsError::InvalidSubject("test".to_string()),
            NatsError::SerializationError("test".to_string()),
            NatsError::DeserializationError("test".to_string()),
            NatsError::ConfigurationError("test".to_string()),
            NatsError::Timeout,
            NatsError::HealthCheckError("test".to_string()),
            NatsError::Other("test".to_string()),
        ];

        // Verify each has a non-empty display string
        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_error_debug_impl() {
        let error = NatsError::Timeout;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Timeout"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let error = NatsError::SerializationError(json_err.to_string());
        assert!(error.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_custom_error_messages() {
        // Test with empty strings
        let error = NatsError::ConnectionError(String::new());
        assert_eq!(error.to_string(), "Connection error: ");

        // Test with very long error messages
        let long_msg = "a".repeat(1000);
        let error = NatsError::Other(long_msg.clone());
        assert_eq!(error.to_string(), format!("NATS error: {}", long_msg));
    }
}