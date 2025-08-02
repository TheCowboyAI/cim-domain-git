// Copyright 2025 Cowboy AI, LLC.

//! Subscribers for commands and events

use async_nats::{Client, Message, Subscriber};
use bytes::Bytes;
use chrono::Utc;
use futures::stream::StreamExt;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{
    command_ack::{AckPublisher, CommandTracker},
    error::{NatsError, Result},
    subject::{GitSubject, MessageType},
};

/// Trait for handling commands
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// The command type this handler processes
    type Command: DeserializeOwned + Send + Sync;

    /// The result type returned by the handler
    type Result: serde::Serialize + Send + Sync;

    /// Handle a command
    async fn handle(&self, command: Self::Command) -> Result<Self::Result>;

    /// Get the command type name
    fn command_type(&self) -> &'static str;
}

/// Trait for handling events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// The event type this handler processes
    type Event: DeserializeOwned + Send + Sync;

    /// Handle an event
    async fn handle(&self, event: Self::Event) -> Result<()>;

    /// Get the event type name
    fn event_type(&self) -> &'static str;
}

/// Command subscriber that processes commands from NATS
pub struct CommandSubscriber {
    client: Client,
    handlers: Arc<
        RwLock<
            Vec<
                Box<
                    dyn CommandHandler<Command = serde_json::Value, Result = serde_json::Value>
                        + Send
                        + Sync,
                >,
            >,
        >,
    >,
    ack_publisher: Arc<AckPublisher>,
}

impl CommandSubscriber {
    /// Create a new command subscriber
    pub fn new(client: Client, handler_id: String) -> Self {
        let ack_publisher = Arc::new(AckPublisher::new(client.clone(), handler_id));
        Self {
            client,
            handlers: Arc::new(RwLock::new(Vec::new())),
            ack_publisher,
        }
    }

    /// Register a command handler
    pub async fn register_handler<H>(&self, handler: H)
    where
        H: CommandHandler + Send + Sync + 'static,
        H::Command: DeserializeOwned + Send + Sync + 'static,
        H::Result: serde::Serialize + Send + Sync + 'static,
    {
        let wrapped = Box::new(TypeErasedCommandHandler::new(handler));
        let mut handlers = self.handlers.write().await;
        handlers.push(wrapped);
    }

    /// Start subscribing to commands
    pub async fn start(&self) -> Result<()> {
        let subject = GitSubject::wildcard(MessageType::Command);
        info!("Subscribing to commands on subject: {}", subject);

        let subscriber = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

        self.process_messages(subscriber).await
    }

    /// Process incoming messages
    async fn process_messages(&self, mut subscriber: Subscriber) -> Result<()> {
        while let Some(message) = subscriber.next().await {
            let handlers = self.handlers.clone();
            let ack_publisher = self.ack_publisher.clone();
            let client = self.client.clone();

            // Spawn a task to handle the message
            tokio::spawn(async move {
                if let Err(e) = Self::handle_message(message, handlers, ack_publisher, client).await {
                    error!("Error handling command: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single message
    async fn handle_message(
        message: Message,
        handlers: Arc<
            RwLock<
                Vec<
                    Box<
                        dyn CommandHandler<Command = serde_json::Value, Result = serde_json::Value>
                            + Send
                            + Sync,
                    >,
                >,
            >,
        >,
        ack_publisher: Arc<AckPublisher>,
        client: Client,
    ) -> Result<()> {
        let subject = message.subject.as_str();
        debug!("Received command on subject: {}", subject);

        // Extract command ID from headers
        let command_id = message
            .headers
            .as_ref()
            .and_then(|h| h.get("X-Command-ID"))
            .and_then(|v| Some(v.as_str()))
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        // Create command tracker
        let tracker = CommandTracker::new((*ack_publisher).clone(), command_id);

        // Send received acknowledgment
        if let Err(e) = tracker.received().await {
            warn!("Failed to send received ack: {}", e);
        }

        // Extract command type from headers or subject
        let command_type = message
            .headers
            .as_ref()
            .and_then(|h| h.get("X-Command-Type"))
            .and_then(|v| Some(v.as_str()))
            .unwrap_or("Unknown");

        // Deserialize the command
        let command: serde_json::Value = match serde_json::from_slice(&message.payload) {
            Ok(cmd) => cmd,
            Err(e) => {
                let error = format!("Failed to deserialize command: {}", e);
                let _ = tracker.rejected(error.clone()).await;
                return Err(NatsError::DeserializationError(error));
            }
        };

        // Find a handler for this command type
        let handlers = handlers.read().await;
        let mut found_handler = false;

        #[allow(unused_assignments)]
        for handler in handlers.iter() {
            if handler.command_type() == command_type {
                found_handler = true;

                // Execute with tracking
                let start_time = Utc::now();
                let _ = tracker.processing().await;

                match handler.handle(command.clone()).await {
                    Ok(result) => {
                        // Send completed acknowledgment
                        if let Err(e) = tracker.completed().await {
                            warn!("Failed to send completed ack: {}", e);
                        }

                        // If there's a reply subject, send the result
                        if let Some(reply) = message.reply {
                            let payload = serde_json::to_vec(&result)
                                .map_err(|e| NatsError::SerializationError(e.to_string()))?;

                            client
                                .publish(reply, Bytes::from(payload))
                                .await
                                .map_err(|e| NatsError::PublishError(e.to_string()))?;
                        }

                        info!(
                            "Successfully handled command: {} ({}ms)",
                            command_type,
                            (Utc::now() - start_time).num_milliseconds()
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        // Send failed acknowledgment
                        let error_msg = format!("Handler error: {}", e);
                        if let Err(ack_err) = tracker.failed(error_msg.clone()).await {
                            warn!("Failed to send failed ack: {}", ack_err);
                        }

                        warn!("Handler error for command {}: {}", command_type, e);
                        return Err(e);
                    }
                }
            }
        }

        if !found_handler {
            let reason = format!("No handler found for command type: {}", command_type);
            let _ = tracker.rejected(reason.clone()).await;
            warn!("{}", reason);
        }

        Ok(())
    }
}

/// Event subscriber that processes events from NATS
pub struct EventSubscriber {
    client: Client,
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler<Event = serde_json::Value> + Send + Sync>>>>,
}

impl EventSubscriber {
    /// Create a new event subscriber
    pub fn new(client: Client) -> Self {
        Self {
            client,
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register an event handler
    pub async fn register_handler<H>(&self, handler: H)
    where
        H: EventHandler + Send + Sync + 'static,
        H::Event: DeserializeOwned + Send + Sync + 'static,
    {
        let wrapped = Box::new(TypeErasedEventHandler::new(handler));
        let mut handlers = self.handlers.write().await;
        handlers.push(wrapped);
    }

    /// Start subscribing to events
    pub async fn start(&self) -> Result<()> {
        let subject = GitSubject::wildcard(MessageType::Event);
        info!("Subscribing to events on subject: {}", subject);

        let subscriber = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

        self.process_messages(subscriber).await
    }

    /// Process incoming messages
    async fn process_messages(&self, mut subscriber: Subscriber) -> Result<()> {
        while let Some(message) = subscriber.next().await {
            let handlers = self.handlers.clone();

            // Spawn a task to handle the message
            tokio::spawn(async move {
                if let Err(e) = Self::handle_message(message, handlers).await {
                    error!("Error handling event: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single message
    async fn handle_message(
        message: Message,
        handlers: Arc<RwLock<Vec<Box<dyn EventHandler<Event = serde_json::Value> + Send + Sync>>>>,
    ) -> Result<()> {
        let subject = message.subject.as_str();
        debug!("Received event on subject: {}", subject);

        // Extract event type from headers or payload
        let event_type = message
            .headers
            .as_ref()
            .and_then(|h| h.get("X-Event-Type"))
            .map(|v| v.as_str().to_string())
            .unwrap_or_else(|| {
                // Try to extract from payload
                serde_json::from_slice::<serde_json::Value>(&message.payload)
                    .ok()
                    .and_then(|v| v.get("event_type").and_then(|e| e.as_str()).map(|s| s.to_string()))
                    .unwrap_or_else(|| "Unknown".to_string())
            });

        // Deserialize the event
        let event: serde_json::Value = serde_json::from_slice(&message.payload)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

        // Find handlers for this event type
        let handlers = handlers.read().await;
        let mut handled = false;

        for handler in handlers.iter() {
            if handler.event_type() == event_type {
                match handler.handle(event.clone()).await {
                    Ok(_) => {
                        debug!("Successfully handled event: {}", event_type);
                        handled = true;
                    }
                    Err(e) => {
                        warn!("Handler error for event {}: {}", event_type, e);
                    }
                }
            }
        }

        if !handled {
            debug!("No handler found for event type: {}", event_type);
        }

        Ok(())
    }
}

/// Type-erased command handler wrapper
struct TypeErasedCommandHandler<H: CommandHandler> {
    handler: H,
    _phantom: std::marker::PhantomData<(H::Command, H::Result)>,
}

impl<H: CommandHandler> TypeErasedCommandHandler<H> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<H> CommandHandler for TypeErasedCommandHandler<H>
where
    H: CommandHandler + Send + Sync,
    H::Command: DeserializeOwned + Send + Sync,
    H::Result: serde::Serialize + Send + Sync,
{
    type Command = serde_json::Value;
    type Result = serde_json::Value;

    async fn handle(&self, command: Self::Command) -> Result<Self::Result> {
        // Deserialize to the concrete type
        let typed_command: H::Command = serde_json::from_value(command)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

        // Handle with the wrapped handler
        let result = self.handler.handle(typed_command).await?;

        // Serialize the result
        let json_result = serde_json::to_value(result)
            .map_err(|e| NatsError::SerializationError(e.to_string()))?;

        Ok(json_result)
    }

    fn command_type(&self) -> &'static str {
        self.handler.command_type()
    }
}

/// Type-erased event handler wrapper
struct TypeErasedEventHandler<H: EventHandler> {
    handler: H,
    _phantom: std::marker::PhantomData<H::Event>,
}

impl<H: EventHandler> TypeErasedEventHandler<H> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<H> EventHandler for TypeErasedEventHandler<H>
where
    H: EventHandler + Send + Sync,
    H::Event: DeserializeOwned + Send + Sync,
{
    type Event = serde_json::Value;

    async fn handle(&self, event: Self::Event) -> Result<()> {
        // Deserialize to the concrete type
        let typed_event: H::Event = serde_json::from_value(event)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

        // Handle with the wrapped handler
        self.handler.handle(typed_event).await
    }

    fn event_type(&self) -> &'static str {
        self.handler.event_type()
    }
}
