// Copyright 2025 Cowboy AI, LLC.

//! Error types for NATS operations

use thiserror::Error;

/// Errors that can occur during NATS operations
#[derive(Debug, Error)]
pub enum NatsError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Publish error
    #[error("Publish error: {0}")]
    PublishError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid subject
    #[error("Invalid subject: {0}")]
    InvalidSubject(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscoveryError(String),

    /// Health check error
    #[error("Health check error: {0}")]
    HealthCheckError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Other errors
    #[error("NATS error: {0}")]
    Other(String),
}

impl From<async_nats::Error> for NatsError {
    fn from(err: async_nats::Error) -> Self {
        // Check error message for common patterns
        let err_str = err.to_string();
        if err_str.contains("timed out") || err_str.contains("timeout") {
            NatsError::Timeout
        } else {
            NatsError::Other(err_str)
        }
    }
}

impl From<serde_json::Error> for NatsError {
    fn from(err: serde_json::Error) -> Self {
        NatsError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for NatsError {
    fn from(err: std::io::Error) -> Self {
        NatsError::Other(err.to_string())
    }
}

/// Result type for NATS operations
pub type Result<T> = std::result::Result<T, NatsError>;
