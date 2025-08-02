// Copyright 2025 Cowboy AI, LLC.

//! NATS integration for the Git domain
//!
//! This module provides NATS messaging capabilities for the Git domain,
//! enabling distributed command processing and event streaming.

pub mod client;
pub mod command_ack;
pub mod config;
pub mod error;
pub mod event_store;
pub mod health;
pub mod projection;
pub mod publisher;
pub mod subject;
pub mod subscriber;
// pub mod tracing; // Temporarily disabled due to OpenSSL dependency

#[cfg(test)]
mod subject_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod subscriber_tests;

// Re-export commonly used types
pub use client::NatsClient;
pub use command_ack::{AckPublisher, AckStatus, AckSubscriber, CommandAck, CommandTracker};
pub use config::{NatsAuth, NatsConfig, NatsTls};
pub use error::{NatsError, Result};
pub use event_store::{ConsumerPosition, EventStore, EventStoreConfig, StreamInfo};
pub use health::{HealthService, ServiceDiscovery, ServiceInfo};
pub use projection::{Projection, ProjectionManager, ProjectionStatus, RepositoryStatsProjection};
pub use publisher::{EventPublisher, EventPublishing};
pub use subject::{Aggregate, CommandAction, EventAction, GitSubject, QueryAction, SubjectMapper};
pub use subscriber::{CommandHandler, CommandSubscriber, EventHandler, EventSubscriber};
// pub use tracing::{TracingConfig, TracingManager, TraceContext, TracedCommand, TracedEvent};
