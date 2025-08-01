// Copyright 2025 Cowboy AI, LLC.

//! Subscribers for commands and events

use async_nats::{Client, Message, Subscriber};
use bytes::Bytes;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{
    error::{NatsError, Result},
    subject::{GitSubject, MessageType},
};

/// Trait for handling commands
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// The command type this handler processes
    type Command: DeserializeOwned + Send;
    
    /// The result type returned by the handler
    type Result: serde::Serialize + Send;
    
    /// Handle a command
    async fn handle(&self, command: Self::Command) -> Result<Self::Result>;
    
    /// Get the command type name
    fn command_type(&self) -> &'static str;
}

/// Trait for handling events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// The event type this handler processes
    type Event: DeserializeOwned + Send;
    
    /// Handle an event
    async fn handle(&self, event: Self::Event) -> Result<()>;
    
    /// Get the event type name
    fn event_type(&self) -> &'static str;
}

/// Command subscriber that processes commands from NATS
pub struct CommandSubscriber {
    client: Client,
    handlers: Arc<RwLock<Vec<Box<dyn CommandHandler<Command = serde_json::Value, Result = serde_json::Value> + Send + Sync>>>>,
}

impl CommandSubscriber {
    /// Create a new command subscriber
    pub fn new(client: Client) -> Self {
        Self {
            client,
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a command handler
    pub async fn register_handler<H>(&self, handler: H)
    where
        H: CommandHandler + Send + Sync + 'static,
        H::Command: DeserializeOwned + Send + 'static,
        H::Result: serde::Serialize + Send + 'static,
    {
        let wrapped = Box::new(TypeErasedCommandHandler::new(handler));
        let mut handlers = self.handlers.write().await;
        handlers.push(wrapped);
    }
    
    /// Start subscribing to commands
    pub async fn start(&self) -> Result<()> {
        let subject = GitSubject::wildcard(MessageType::Command);
        info!("Subscribing to commands on subject: {}", subject);
        
        let subscriber = self.client
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
                    error!("Error handling command: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    /// Handle a single message
    async fn handle_message(
        message: Message,
        handlers: Arc<RwLock<Vec<Box<dyn CommandHandler<Command = serde_json::Value, Result = serde_json::Value> + Send + Sync>>>>,
    ) -> Result<()> {
        let subject = message.subject.as_str();
        debug!("Received command on subject: {}", subject);
        
        // Extract command type from headers or subject
        let command_type = message.headers
            .as_ref()
            .and_then(|h| h.get("X-Command-Type"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
        // Deserialize the command
        let command: serde_json::Value = serde_json::from_slice(&message.payload)
            .map_err(|e| NatsError::DeserializationError(e.to_string()))?;
        
        // Find a handler for this command type
        let handlers = handlers.read().await;
        for handler in handlers.iter() {
            if handler.command_type() == command_type {
                match handler.handle(command.clone()).await {
                    Ok(result) => {
                        // If there's a reply subject, send the result
                        if let Some(reply) = message.reply {
                            let payload = serde_json::to_vec(&result)
                                .map_err(|e| NatsError::SerializationError(e.to_string()))?;
                            
                            message.client
                                .publish(reply, Bytes::from(payload))
                                .await
                                .map_err(|e| NatsError::PublishError(e.to_string()))?;
                        }
                        
                        info!("Successfully handled command: {}", command_type);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Handler error for command {}: {}", command_type, e);
                    }
                }
            }
        }
        
        warn!("No handler found for command type: {}", command_type);
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
        H::Event: DeserializeOwned + Send + 'static,
    {
        let wrapped = Box::new(TypeErasedEventHandler::new(handler));
        let mut handlers = self.handlers.write().await;
        handlers.push(wrapped);
    }
    
    /// Start subscribing to events
    pub async fn start(&self) -> Result<()> {
        let subject = GitSubject::wildcard(MessageType::Event);
        info!("Subscribing to events on subject: {}", subject);
        
        let subscriber = self.client
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
        let event_type = message.headers
            .as_ref()
            .and_then(|h| h.get("X-Event-Type"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                // Try to extract from payload
                serde_json::from_slice::<serde_json::Value>(&message.payload)
                    .ok()
                    .and_then(|v| v.get("event_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
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
    H::Command: DeserializeOwned + Send,
    H::Result: serde::Serialize + Send,
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
    H::Event: DeserializeOwned + Send,
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