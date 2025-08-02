// Copyright 2025 Cowboy AI, LLC.

//! Command acknowledgment handling for reliable command processing
//!
//! This module provides acknowledgment capabilities for commands to ensure
//! reliable processing in distributed systems.

use async_nats::Client;
use futures::StreamExt;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

use super::error::{NatsError, Result};

/// Command acknowledgment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    /// Command was received and queued for processing
    Received,
    /// Command is being processed
    Processing,
    /// Command completed successfully
    Completed,
    /// Command failed with error
    Failed,
    /// Command was rejected (invalid, unauthorized, etc.)
    Rejected,
    /// Command processing timed out
    TimedOut,
}

/// Command acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAck {
    /// Command ID being acknowledged
    pub command_id: Uuid,

    /// Acknowledgment status
    pub status: AckStatus,

    /// Handler that sent the acknowledgment
    pub handler_id: String,

    /// When the acknowledgment was sent
    pub timestamp: DateTime<Utc>,

    /// Optional message
    pub message: Option<String>,

    /// Error details if failed
    pub error: Option<String>,

    /// Processing duration in milliseconds (for completed/failed)
    pub duration_ms: Option<u64>,
}

impl CommandAck {
    /// Create a new acknowledgment
    pub fn new(command_id: Uuid, status: AckStatus, handler_id: String) -> Self {
        Self {
            command_id,
            status,
            handler_id,
            timestamp: Utc::now(),
            message: None,
            error: None,
            duration_ms: None,
        }
    }

    /// Create a received acknowledgment
    pub fn received(command_id: Uuid, handler_id: String) -> Self {
        Self::new(command_id, AckStatus::Received, handler_id)
    }

    /// Create a processing acknowledgment
    pub fn processing(command_id: Uuid, handler_id: String) -> Self {
        Self::new(command_id, AckStatus::Processing, handler_id)
    }

    /// Create a completed acknowledgment
    pub fn completed(command_id: Uuid, handler_id: String, duration_ms: u64) -> Self {
        let mut ack = Self::new(command_id, AckStatus::Completed, handler_id);
        ack.duration_ms = Some(duration_ms);
        ack
    }

    /// Create a failed acknowledgment
    pub fn failed(command_id: Uuid, handler_id: String, error: String, duration_ms: u64) -> Self {
        let mut ack = Self::new(command_id, AckStatus::Failed, handler_id);
        ack.error = Some(error);
        ack.duration_ms = Some(duration_ms);
        ack
    }

    /// Create a rejected acknowledgment
    pub fn rejected(command_id: Uuid, handler_id: String, reason: String) -> Self {
        let mut ack = Self::new(command_id, AckStatus::Rejected, handler_id);
        ack.message = Some(reason);
        ack
    }

    /// Add a message to the acknowledgment
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

/// Command acknowledgment publisher
#[derive(Clone)]
pub struct AckPublisher {
    /// NATS client
    client: Client,

    /// Handler ID for this instance
    handler_id: String,
}

impl AckPublisher {
    /// Create a new acknowledgment publisher
    pub fn new(client: Client, handler_id: String) -> Self {
        Self { client, handler_id }
    }

    /// Publish an acknowledgment
    pub async fn publish(&self, ack: &CommandAck) -> Result<()> {
        let subject = format!("git.ack.{}", ack.command_id);

        let payload =
            serde_json::to_vec(ack).map_err(|e| NatsError::SerializationError(e.to_string()))?;

        self.client
            .publish(subject, Bytes::from(payload))
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        debug!(
            "Published {} acknowledgment for command {}",
            ack.status as u8, ack.command_id
        );

        Ok(())
    }

    /// Send received acknowledgment
    pub async fn ack_received(&self, command_id: Uuid) -> Result<()> {
        let ack = CommandAck::received(command_id, self.handler_id.clone());
        self.publish(&ack).await
    }

    /// Send processing acknowledgment
    pub async fn ack_processing(&self, command_id: Uuid) -> Result<()> {
        let ack = CommandAck::processing(command_id, self.handler_id.clone());
        self.publish(&ack).await
    }

    /// Send completed acknowledgment
    pub async fn ack_completed(&self, command_id: Uuid, start_time: DateTime<Utc>) -> Result<()> {
        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;
        let ack = CommandAck::completed(command_id, self.handler_id.clone(), duration_ms);
        self.publish(&ack).await
    }

    /// Send failed acknowledgment
    pub async fn ack_failed(
        &self,
        command_id: Uuid,
        error: String,
        start_time: DateTime<Utc>,
    ) -> Result<()> {
        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;
        let ack = CommandAck::failed(command_id, self.handler_id.clone(), error, duration_ms);
        self.publish(&ack).await
    }

    /// Send rejected acknowledgment
    pub async fn ack_rejected(&self, command_id: Uuid, reason: String) -> Result<()> {
        let ack = CommandAck::rejected(command_id, self.handler_id.clone(), reason);
        self.publish(&ack).await
    }
}

/// Command acknowledgment subscriber for monitoring command processing
pub struct AckSubscriber {
    /// NATS client
    client: Client,
}

impl AckSubscriber {
    /// Create a new acknowledgment subscriber
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Subscribe to acknowledgments for a specific command
    pub async fn subscribe_to_command(
        &self,
        command_id: Uuid,
        timeout: Duration,
    ) -> Result<Vec<CommandAck>> {
        let subject = format!("git.ack.{}", command_id);
        let mut subscription = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

        let mut acks = Vec::new();
        let deadline = Utc::now() + chrono::Duration::from_std(timeout).unwrap();

        while Utc::now() < deadline {
            let timeout_duration = (deadline - Utc::now())
                .to_std()
                .unwrap_or(Duration::from_millis(1));

            match tokio::time::timeout(timeout_duration, subscription.next()).await {
                Ok(Some(message)) => {
                    if let Ok(ack) = serde_json::from_slice::<CommandAck>(&message.payload) {
                        debug!(
                            "Received {} ack for command {}",
                            ack.status as u8, command_id
                        );

                        let is_terminal = matches!(
                            ack.status,
                            AckStatus::Completed | AckStatus::Failed | AckStatus::Rejected
                        );

                        acks.push(ack);

                        if is_terminal {
                            break;
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    warn!("Timeout waiting for command {} acknowledgment", command_id);
                    break;
                }
            }
        }

        Ok(acks)
    }

    /// Subscribe to all acknowledgments
    pub async fn subscribe_all(&self) -> Result<async_nats::Subscriber> {
        self.client
            .subscribe("git.ack.>")
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))
    }
}

/// Command execution tracker that combines acknowledgment with execution
pub struct CommandTracker {
    /// Acknowledgment publisher
    ack_publisher: AckPublisher,

    /// Command start time
    start_time: DateTime<Utc>,

    /// Command ID being tracked
    command_id: Uuid,
}

impl CommandTracker {
    /// Create a new command tracker
    pub fn new(ack_publisher: AckPublisher, command_id: Uuid) -> Self {
        Self {
            ack_publisher,
            start_time: Utc::now(),
            command_id,
        }
    }

    /// Mark command as received
    pub async fn received(&self) -> Result<()> {
        self.ack_publisher.ack_received(self.command_id).await
    }

    /// Mark command as processing
    pub async fn processing(&self) -> Result<()> {
        self.ack_publisher.ack_processing(self.command_id).await
    }

    /// Mark command as completed
    pub async fn completed(&self) -> Result<()> {
        self.ack_publisher
            .ack_completed(self.command_id, self.start_time)
            .await
    }

    /// Mark command as failed
    pub async fn failed(&self, error: String) -> Result<()> {
        self.ack_publisher
            .ack_failed(self.command_id, error, self.start_time)
            .await
    }

    /// Mark command as rejected
    pub async fn rejected(&self, reason: String) -> Result<()> {
        self.ack_publisher
            .ack_rejected(self.command_id, reason)
            .await
    }

    /// Execute a command with automatic acknowledgment
    pub async fn execute<F, T, E>(self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::error::Error,
    {
        // Send processing acknowledgment
        self.processing().await?;

        // Execute the command
        match f.await {
            Ok(result) => {
                // Send completed acknowledgment
                self.completed().await?;
                Ok(result)
            }
            Err(error) => {
                // Send failed acknowledgment
                self.failed(error.to_string()).await?;
                Err(NatsError::Other(format!(
                    "Command execution failed: {}",
                    error
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ack_creation() {
        let command_id = Uuid::new_v4();
        let handler_id = "test-handler".to_string();

        let ack = CommandAck::received(command_id, handler_id.clone());
        assert_eq!(ack.status, AckStatus::Received);
        assert_eq!(ack.command_id, command_id);
        assert_eq!(ack.handler_id, handler_id);
        assert!(ack.error.is_none());
        assert!(ack.duration_ms.is_none());
    }

    #[test]
    fn test_ack_with_message() {
        let command_id = Uuid::new_v4();
        let handler_id = "test-handler".to_string();

        let ack = CommandAck::completed(command_id, handler_id, 100)
            .with_message("Command processed successfully".to_string());

        assert_eq!(ack.status, AckStatus::Completed);
        assert_eq!(ack.duration_ms, Some(100));
        assert_eq!(
            ack.message,
            Some("Command processed successfully".to_string())
        );
    }

    #[test]
    fn test_failed_ack() {
        let command_id = Uuid::new_v4();
        let handler_id = "test-handler".to_string();
        let error = "Invalid repository URL".to_string();

        let ack = CommandAck::failed(command_id, handler_id, error.clone(), 50);

        assert_eq!(ack.status, AckStatus::Failed);
        assert_eq!(ack.error, Some(error));
        assert_eq!(ack.duration_ms, Some(50));
    }
}
