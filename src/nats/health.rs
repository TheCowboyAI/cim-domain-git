// Copyright 2025 Cowboy AI, LLC.

//! Health check and service discovery for Git domain

use async_nats::Client;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

use super::error::{NatsError, Result};

/// Service information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID (unique instance identifier)
    pub id: String,

    /// Service name (e.g., "git-domain")
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Service endpoints
    pub endpoints: Vec<String>,

    /// Service metadata
    pub metadata: HashMap<String, String>,

    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,

    /// Service status
    pub status: ServiceStatus,
}

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Service is healthy and running
    Healthy,

    /// Service is degraded but operational
    Degraded,

    /// Service is unhealthy
    Unhealthy,

    /// Service is shutting down
    Stopping,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub status: ServiceStatus,

    /// Individual component checks
    pub checks: HashMap<String, ComponentHealth>,

    /// Timestamp of the check
    pub timestamp: DateTime<Utc>,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Component status
    pub status: ServiceStatus,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<HashMap<String, f64>>,
}

/// Health service for monitoring and service discovery
pub struct HealthService {
    client: Client,
    service_info: ServiceInfo,
    health_checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
}

impl HealthService {
    /// Create a new health service
    pub fn new(client: Client, service_info: ServiceInfo) -> Self {
        Self {
            client,
            service_info,
            health_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a health check
    pub async fn register_check(&self, name: String, check: Box<dyn HealthCheck + Send + Sync>) {
        let mut checks = self.health_checks.write().await;
        checks.insert(name, check);
    }

    /// Start the health service (heartbeat and service discovery)
    pub async fn start(&self) -> Result<()> {
        // Start heartbeat
        let heartbeat_handle = self.start_heartbeat();

        // Start health check endpoint
        let health_check_handle = self.start_health_endpoint();

        // Wait for both tasks
        tokio::try_join!(heartbeat_handle, health_check_handle)
            .map_err(|e| NatsError::Other(format!("Task join error: {}", e)))?;

        Ok(())
    }

    /// Start heartbeat publishing
    fn start_heartbeat(&self) -> tokio::task::JoinHandle<Result<()>> {
        let client = self.client.clone();
        let mut service_info = self.service_info.clone();
        let subject = format!("_SERVICES.{}.{}", service_info.name, service_info.id);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));

            loop {
                ticker.tick().await;

                // Update heartbeat timestamp
                service_info.last_heartbeat = Utc::now();

                // Publish service info
                let payload = serde_json::to_vec(&service_info)
                    .map_err(|e| NatsError::SerializationError(e.to_string()))?;

                if let Err(e) = client.publish(subject.clone(), payload.into()).await {
                    error!("Failed to publish heartbeat: {}", e);
                } else {
                    debug!("Published heartbeat");
                }
            }
        })
    }

    /// Start health check endpoint
    fn start_health_endpoint(&self) -> tokio::task::JoinHandle<Result<()>> {
        let client = self.client.clone();
        let service_name = self.service_info.name.clone();
        let health_checks = self.health_checks.clone();
        let subject = format!("_HEALTH.{}", service_name);

        tokio::spawn(async move {
            let mut subscriber = client
                .subscribe(subject)
                .await
                .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

            info!("Health check endpoint started");

            while let Some(message) = subscriber.next().await {
                let checks = health_checks.clone();
                let client_clone = client.clone();

                // Handle health check request
                tokio::spawn(async move {
                    let result = Self::perform_health_check(checks).await;

                    if let Some(reply) = message.reply {
                        let payload =
                            serde_json::to_vec(&result).unwrap_or_else(|_| b"{}".to_vec());

                        if let Err(e) = client_clone.publish(reply, payload.into()).await {
                            error!("Failed to send health check response: {}", e);
                        }
                    }
                });
            }

            Ok(())
        })
    }

    /// Perform health checks
    async fn perform_health_check(
        checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
    ) -> HealthCheckResult {
        let mut results = HashMap::new();
        let mut overall_status = ServiceStatus::Healthy;

        let checks = checks.read().await;

        for (name, check) in checks.iter() {
            let component_health = check.check().await;

            // Update overall status based on component status
            match component_health.status {
                ServiceStatus::Unhealthy => overall_status = ServiceStatus::Unhealthy,
                ServiceStatus::Degraded => {
                    if overall_status == ServiceStatus::Healthy {
                        overall_status = ServiceStatus::Degraded;
                    }
                }
                _ => {}
            }

            results.insert(name.clone(), component_health);
        }

        HealthCheckResult {
            status: overall_status,
            checks: results,
            timestamp: Utc::now(),
        }
    }
}

/// Trait for implementing health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> ComponentHealth;
}

/// Service discovery client
pub struct ServiceDiscovery {
    client: Client,
    services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
}

impl ServiceDiscovery {
    /// Create a new service discovery client
    pub fn new(client: Client) -> Self {
        Self {
            client,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Discover services of a given type
    pub async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInfo>> {
        let subject = format!("_SERVICES.{}.>", service_name);

        // Request service info
        let response = self
            .client
            .request(subject, vec![].into())
            .await
            .map_err(|e| NatsError::Other(format!("Service discovery failed: {}", e)))?;

        // Parse response
        let service_info: ServiceInfo = serde_json::from_slice(&response.payload)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

        // Update cache
        let mut services = self.services.write().await;
        services
            .entry(service_name.to_string())
            .or_insert_with(Vec::new)
            .push(service_info.clone());

        Ok(vec![service_info])
    }

    /// Get cached services
    pub async fn get_cached(&self, service_name: &str) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        services.get(service_name).cloned().unwrap_or_default()
    }

    /// Check health of a specific service
    pub async fn check_health(&self, service_name: &str) -> Result<HealthCheckResult> {
        let subject = format!("_HEALTH.{}", service_name);

        let response = self
            .client
            .request(subject, vec![].into())
            .await
            .map_err(|e| NatsError::Other(format!("Health check failed: {}", e)))?;

        let result: HealthCheckResult = serde_json::from_slice(&response.payload)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

        Ok(result)
    }
}

/// NATS connection health check
pub struct NatsHealthCheck {
    client: Client,
}

impl NatsHealthCheck {
    /// Create a new NATS health check with the given client
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HealthCheck for NatsHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let status = if self.client.connection_state() == async_nats::connection::State::Connected {
            ServiceStatus::Healthy
        } else {
            ServiceStatus::Unhealthy
        };

        ComponentHealth {
            name: "nats_connection".to_string(),
            status,
            message: Some(format!(
                "Connection state: {:?}",
                self.client.connection_state()
            )),
            metrics: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            id: Uuid::new_v4().to_string(),
            name: "git-domain".to_string(),
            version: "0.1.0".to_string(),
            description: Some("Git domain service".to_string()),
            endpoints: vec!["git.>".to_string()],
            metadata: HashMap::new(),
            last_heartbeat: Utc::now(),
            status: ServiceStatus::Healthy,
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.status, deserialized.status);
    }
}
