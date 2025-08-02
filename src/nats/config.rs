// Copyright 2025 Cowboy AI, LLC.

//! Configuration for NATS client

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// NATS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL (e.g., "nats://localhost:4222")
    pub url: String,

    /// Service information
    pub service: ServiceConfig,

    /// Authentication configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<NatsAuth>,

    /// TLS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<NatsTls>,

    /// Connection retry configuration
    pub retry: RetryConfig,

    /// JetStream configuration
    pub jetstream: JetStreamConfig,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            service: ServiceConfig::default(),
            auth: None,
            tls: None,
            retry: RetryConfig::default(),
            jetstream: JetStreamConfig::default(),
        }
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name (e.g., "git-domain")
    pub name: String,

    /// Service version
    pub version: String,

    /// Service instance ID (auto-generated if not provided)
    pub instance_id: String,

    /// Service description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "git-domain".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: format!("git-domain-{}", uuid::Uuid::new_v4()),
            description: Some("CIM Git Domain Service".to_string()),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAuth {
    /// Username for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Token for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// NKey seed for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey: Option<String>,

    /// JWT for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsTls {
    /// Path to CA certificate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert: Option<String>,

    /// Path to client certificate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,

    /// Path to client key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,

    /// Skip TLS verification (dangerous!)
    #[serde(default)]
    pub insecure_skip_verify: bool,
}

/// Connection retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of reconnection attempts
    pub max_reconnects: Option<u32>,

    /// Delay between reconnection attempts (ms)
    pub reconnect_delay_ms: u64,

    /// Connection timeout (ms)
    pub connect_timeout_ms: u64,

    /// Request timeout (ms)
    pub request_timeout_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_reconnects: None, // Infinite
            reconnect_delay_ms: 1000,
            connect_timeout_ms: 5000,
            request_timeout_ms: 30000,
        }
    }
}

/// JetStream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Whether to enable JetStream
    pub enabled: bool,

    /// Stream name for events
    pub event_stream: String,

    /// Stream name for commands
    pub command_stream: String,

    /// Maximum age for messages (seconds)
    pub max_age_secs: u64,

    /// Maximum number of messages in the stream
    pub max_messages: i64,

    /// Storage type (file or memory)
    pub storage: StorageType,

    /// Number of replicas
    pub num_replicas: usize,
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            event_stream: "GIT_EVENTS".to_string(),
            command_stream: "GIT_COMMANDS".to_string(),
            max_age_secs: 7 * 24 * 60 * 60, // 7 days
            max_messages: 10_000_000,
            storage: StorageType::File,
            num_replicas: 1,
        }
    }
}

/// Storage type for JetStream
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    File,
    Memory,
}

impl NatsConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, super::error::NatsError> {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(url) = std::env::var("NATS_URL") {
            config.url = url;
        }

        if let Ok(name) = std::env::var("NATS_SERVICE_NAME") {
            config.service.name = name;
        }

        if let Ok(user) = std::env::var("NATS_USER") {
            let auth = config.auth.get_or_insert(NatsAuth {
                username: None,
                password: None,
                token: None,
                nkey: None,
                jwt: None,
            });
            auth.username = Some(user);
        }

        if let Ok(pass) = std::env::var("NATS_PASSWORD") {
            let auth = config.auth.get_or_insert(NatsAuth {
                username: None,
                password: None,
                token: None,
                nkey: None,
                jwt: None,
            });
            auth.password = Some(pass);
        }

        if let Ok(token) = std::env::var("NATS_TOKEN") {
            let auth = config.auth.get_or_insert(NatsAuth {
                username: None,
                password: None,
                token: None,
                nkey: None,
                jwt: None,
            });
            auth.token = Some(token);
        }

        Ok(config)
    }

    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.retry.request_timeout_ms)
    }

    /// Get connection timeout as Duration
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.retry.connect_timeout_ms)
    }

    /// Get reconnect delay as Duration
    pub fn reconnect_delay(&self) -> Duration {
        Duration::from_millis(self.retry.reconnect_delay_ms)
    }
}
