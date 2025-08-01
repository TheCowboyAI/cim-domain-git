// Copyright 2025 Cowboy AI, LLC.

//! NATS client wrapper for the Git domain

use async_nats::{Client, ConnectOptions};
use tracing::{debug, info, warn};
use std::time::Duration;

use super::{config::NatsConfig, error::{NatsError, Result}};

/// NATS client wrapper with domain-specific configuration
pub struct NatsClient {
    /// The underlying NATS client
    client: Client,
    
    /// Configuration
    config: NatsConfig,
}

impl NatsClient {
    /// Connect to NATS with the given configuration
    pub async fn connect(config: NatsConfig) -> Result<Self> {
        info!("Connecting to NATS at {}", config.url);
        
        let mut options = ConnectOptions::new()
            .name(&config.service.name)
            .retry_on_initial_connect()
            .max_reconnects(config.retry.max_reconnects)
            .reconnect_delay(Duration::from_millis(config.retry.reconnect_delay_ms))
            .connection_timeout(Duration::from_millis(config.retry.connect_timeout_ms));
        
        // Add authentication if configured
        if let Some(auth) = &config.auth {
            if let (Some(user), Some(pass)) = (&auth.username, &auth.password) {
                debug!("Using username/password authentication");
                options = options.user_and_password(user.clone(), pass.clone());
            } else if let Some(token) = &auth.token {
                debug!("Using token authentication");
                options = options.token(token.clone());
            } else if let Some(nkey) = &auth.nkey {
                debug!("Using NKey authentication");
                // Note: async-nats requires parsing the nkey
                // This is a simplified example
                options = options.nkey(nkey.clone());
            }
        }
        
        // Add TLS if configured
        if let Some(tls) = &config.tls {
            debug!("Configuring TLS");
            let mut tls_config = async_nats::ConnectOptions::new();
            
            if let Some(ca_cert) = &tls.ca_cert {
                tls_config = tls_config.add_root_certificate(ca_cert.into());
            }
            
            if let (Some(cert), Some(key)) = (&tls.client_cert, &tls.client_key) {
                tls_config = tls_config.client_cert(cert.into(), key.into());
            }
            
            if tls.insecure_skip_verify {
                warn!("TLS verification disabled - this is insecure!");
                tls_config = tls_config.tls_required(false);
            }
            
            options = tls_config;
        }
        
        // Connect with retry
        let client = async_nats::connect_with_options(&config.url, options)
            .await
            .map_err(|e| NatsError::ConnectionError(e.to_string()))?;
        
        info!("Successfully connected to NATS");
        
        Ok(Self { client, config })
    }
    
    /// Get the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    /// Get the configuration
    pub fn config(&self) -> &NatsConfig {
        &self.config
    }
    
    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        self.client.connection_state() == async_nats::connection::State::Connected
    }
    
    /// Flush all pending operations
    pub async fn flush(&self) -> Result<()> {
        self.client
            .flush()
            .await
            .map_err(|e| NatsError::Other(e.to_string()))
    }
    
    /// Drain the connection (graceful shutdown)
    pub async fn drain(&self) -> Result<()> {
        info!("Draining NATS connection");
        self.client
            .drain()
            .await
            .map_err(|e| NatsError::Other(e.to_string()))?;
        info!("NATS connection drained");
        Ok(())
    }
    
    /// Create a JetStream context if enabled
    pub async fn jetstream(&self) -> Result<async_nats::jetstream::Context> {
        if !self.config.jetstream.enabled {
            return Err(NatsError::ConfigurationError(
                "JetStream is not enabled in configuration".to_string()
            ));
        }
        
        let jetstream = async_nats::jetstream::new(self.client.clone());
        
        // Ensure streams exist
        self.ensure_streams(&jetstream).await?;
        
        Ok(jetstream)
    }
    
    /// Ensure JetStream streams exist
    async fn ensure_streams(&self, jetstream: &async_nats::jetstream::Context) -> Result<()> {
        use async_nats::jetstream::{stream::Config as StreamConfig, stream::StorageType as JsStorageType};
        
        // Convert our storage type to JetStream storage type
        let storage = match self.config.jetstream.storage {
            super::config::StorageType::File => JsStorageType::File,
            super::config::StorageType::Memory => JsStorageType::Memory,
        };
        
        // Event stream configuration
        let event_stream_config = StreamConfig {
            name: self.config.jetstream.event_stream.clone(),
            subjects: vec![format!("{}.event.>", super::subject::DOMAIN)],
            max_age: Duration::from_secs(self.config.jetstream.max_age_secs),
            max_msgs_per_subject: self.config.jetstream.max_msgs_per_subject,
            storage,
            num_replicas: self.config.jetstream.num_replicas,
            ..Default::default()
        };
        
        // Command stream configuration
        let command_stream_config = StreamConfig {
            name: self.config.jetstream.command_stream.clone(),
            subjects: vec![format!("{}.cmd.>", super::subject::DOMAIN)],
            max_age: Duration::from_secs(self.config.jetstream.max_age_secs),
            max_msgs_per_subject: self.config.jetstream.max_msgs_per_subject,
            storage,
            num_replicas: self.config.jetstream.num_replicas,
            ..Default::default()
        };
        
        // Create or update event stream
        match jetstream.get_or_create_stream(event_stream_config).await {
            Ok(_) => info!("Event stream ready: {}", self.config.jetstream.event_stream),
            Err(e) => {
                return Err(NatsError::Other(format!(
                    "Failed to create event stream: {}",
                    e
                )));
            }
        }
        
        // Create or update command stream
        match jetstream.get_or_create_stream(command_stream_config).await {
            Ok(_) => info!("Command stream ready: {}", self.config.jetstream.command_stream),
            Err(e) => {
                return Err(NatsError::Other(format!(
                    "Failed to create command stream: {}",
                    e
                )));
            }
        }
        
        Ok(())
    }
}

impl Drop for NatsClient {
    fn drop(&mut self) {
        // The async-nats client handles cleanup automatically
        debug!("NATS client dropping");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.service.name, "git-domain");
        assert!(config.jetstream.enabled);
    }
    
    #[tokio::test]
    async fn test_config_from_env() {
        std::env::set_var("NATS_URL", "nats://test:4222");
        std::env::set_var("NATS_SERVICE_NAME", "test-service");
        
        let config = NatsConfig::from_env().unwrap();
        assert_eq!(config.url, "nats://test:4222");
        assert_eq!(config.service.name, "test-service");
        
        // Clean up
        std::env::remove_var("NATS_URL");
        std::env::remove_var("NATS_SERVICE_NAME");
    }
}