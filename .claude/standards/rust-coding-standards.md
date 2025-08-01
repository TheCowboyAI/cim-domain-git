# Rust Coding Standards

## Overview

This document establishes standards for developing robust, scalable systems in Rust that integrate Entity Component System (ECS) patterns with Event-Driven Architecture, Domain-Driven Design principles, and NATS messaging.

## Core Language Standards

### Naming Conventions

- **Variables & Functions**: Use `snake_case` consistently
  - Examples: `movement_system`, `player_component`, `health_value`
- **Constants**: Use `SCREAMING_SNAKE_CASE`
  - Examples: `MAX_ENTITIES`, `DEFAULT_HEALTH_VALUE`
- **Types (Structs/Traits)**: Use `PascalCase`
  - Examples: `HealthComponent`, `MovementSystem`, `EventHandler`

### Module Organization

1. **Flatten Public APIs**: Avoid redundant naming patterns
   ```rust
   // Good: Expose as game::Client
   pub use client::Client;
   
   // Avoid: game::client::Client
   ```

2. **Visibility Control**:
   - Use `pub(crate)` for crate-internal items
   - Use `pub(super)` for parent-module-only access
   - Keep implementation details private

### Error Handling

1. **Centralized Error Module**: Create a flattened error export
   ```rust
   // In lib.rs
   pub use error::Error;  // Not game::error::Error
   ```

2. **Component Operation Failures**: Handle gracefully
   - Type mismatches
   - Missing dependencies
   - Resource constraints

3. **System Resilience**: Allow partial failures without crashing the simulation

### Constructor Patterns

1. **Simple Types**: Implement `Default` trait when possible
2. **Complex Components**: Use builder pattern for multiple optional parameters
3. **Validation**: Enforce invariants at construction time

## Entity Component System (ECS) Architecture

### Component Design Principles

1. **Data-Only Structures**: Components hold data, not behavior
   ```rust
   #[derive(Component)]
   struct Health {
       current: f32,
       maximum: f32,
   }
   ```

2. **Single Responsibility**: Each component represents one aspect
3. **Domain Alignment**: Components reflect domain concepts, not technical details
4. **Proper Granularity**: Balance between too fine and too coarse

### Entity Management

1. **Lightweight Identifiers**: Entities are just IDs referencing components
2. **Atomic Creation**: Spawn entities with all required components at once
3. **Lifecycle Management**: Define and enforce component dependencies
4. **Entity Archetypes**: Create templates for common configurations

### System Design

1. **Pure Functions**: Avoid side effects beyond component updates
2. **Precise Queries**: Request only necessary component access
   ```rust
   fn movement_system(
       mut positions: Query<&mut Position>,
       velocities: Query<&Velocity>,
   ) {
       // System logic
   }
   ```

3. **Explicit Ordering**: Define system dependencies clearly
4. **Appropriate Granularity**: One responsibility per system

### Resource Management

- Use for truly global state only
- Keep resources focused and domain-aligned
- Minimize resource access in systems
- Document initialization requirements

## Domain-Driven Design Integration

### Bounded Contexts

1. **Clear Boundaries**: Align with business capabilities
2. **Ubiquitous Language**: Consistent naming within contexts
3. **Module Enforcement**: Use Rust's visibility controls
4. **Event-Based Communication**: Cross-context interaction via NATS only

### Aggregate Design

1. **Consistency Boundaries**: Map to entities or entity collections
2. **Aggregate Roots**: Enforce modifications through root entity
3. **Business Invariants**: Align boundaries with domain rules
4. **Event Publication**: Publish at aggregate level, not component level

### Domain Events

1. **Business Significance**: Only meaningful occurrences
2. **Intent Focus**: `InventoryDepleted` not `ComponentUpdated`
3. **Careful Orchestration**: Evaluate component changes for event worthiness

### Value Objects

1. **Immutability**: Leverage Rust's ownership system
2. **Domain Types**: Prefer `Price` over raw `f64`
3. **Business Equality**: Implement based on domain rules

## Event-Driven Architecture

### Event Design Patterns

1. **Past-Tense Naming**: `CustomerRegistered`, `PaymentProcessed`
2. **Completed Operations**: Events represent finished actions
3. **Balanced Payloads**: Essential information without coupling
4. **Schema Evolution**: Design for additive changes

### NATS Integration

1. **Hierarchical Subjects**: `inventory.stock.depleted`
2. **Async Publish-Subscribe**: Primary communication pattern
3. **Sparse Request-Reply**: Only for immediate feedback needs
4. **Connection Resilience**: Implement reconnection logic

### System Coordination

1. **Event Reaction**: Systems respond to events, not polling
2. **Processing Order**: Define clear constraints when needed
3. **Error Isolation**: Prevent cascade failures
4. **Observability**: Distributed tracing and metrics

## NATS Messaging Standards

### Client Configuration

1. **Robust Retry Logic**: Exponential backoff for connections
2. **Multiple Auth Support**: Token, username/password, certificates
3. **Secure Credentials**: Environment variables or secure stores
4. **Connection Pooling**: Separate connections per bounded context

### Subject Design

1. **Business-Oriented**: `orders.fulfillment.shipped`
2. **Hierarchical Structure**: Enable flexible subscriptions
3. **Judicious Wildcards**: Avoid overly broad patterns
4. **Version Strategy**: Include version in subjects when needed

### JetStream Usage

1. **Selective Persistence**: Only for critical business events
2. **Appropriate Retention**: Based on business requirements
3. **Pull Consumers**: Better flow control and backpressure
4. **Idempotent Processing**: Handle redelivery gracefully

## Testing Standards

### Testing Strategy

1. **Unit Tests**: Individual systems and components
2. **Integration Tests**: Cross-system coordination
3. **Mock NATS**: Test without full infrastructure
4. **Property-Based Testing**: Verify invariants across inputs

### Test Organization

- Mirror source structure
- Test domain logic without ECS/NATS dependencies
- Verify both success and failure scenarios

## Performance Guidelines

### Memory Optimization

1. **Cache-Friendly Layout**: Group related components
2. **Avoid Pointer Chasing**: Optimize tight loops
3. **Archetype Optimization**: Group identical component sets

### System Scheduling

1. **Parallel Execution**: For disjoint component sets
2. **Explicit Dependencies**: When order matters
3. **Message Batching**: For high-volume NATS traffic

## Security Considerations

### Access Control

1. **Subject-Level Security**: Align with domain boundaries
2. **Field-Level Encryption**: For sensitive data
3. **Audit Logging**: Comprehensive event tracking
4. **Access Monitoring**: Detect anomalous patterns

## Error Handling

- Use `Result<T, E>` for fallible operations
- Define custom error types with `thiserror`
- Provide context with error messages
- Use `anyhow` for application-level errors
- Chain errors with proper context using `?` operator
- Avoid `unwrap()` except in tests or with SAFETY comments

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    
    #[error("Entity not found: {id}")]
    NotFound { id: String },
    
    #[error("Aggregate version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },
}

// Error propagation with context
pub fn process_command(cmd: Command) -> Result<Vec<Event>, DomainError> {
    let aggregate = load_aggregate(&cmd.aggregate_id)
        .map_err(|e| DomainError::NotFound { 
            id: cmd.aggregate_id.to_string() 
        })?;
    
    aggregate.handle(cmd)
}
```

## Testing

- Write unit tests for all public APIs
- Use property-based testing with `proptest`
- Test error conditions explicitly
- Use test fixtures for complex data
- Follow AAA pattern: Arrange, Act, Assert
- Use `#[tokio::test]` for async tests
- Mock external dependencies with trait objects

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_domain_logic() {
        // Arrange
        let aggregate = TestFixture::default_aggregate();
        let command = TestFixture::valid_command();
        
        // Act
        let result = aggregate.handle(command);
        
        // Assert
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(events[0], Event::Created { .. });
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        let store = MockEventStore::new();
        let result = store.load_events("test-id").await;
        assert!(result.is_ok());
    }
    
    proptest! {
        #[test]
        fn test_invariants(cmd: Command) {
            let aggregate = GraphAggregate::new();
            let result = aggregate.handle(cmd);
            
            // Invariant: version always increases
            if result.is_ok() {
                assert!(aggregate.version() > 0);
            }
        }
    }
}
```

## Documentation

- Document all public APIs with rustdoc
- Include examples in documentation
- Use `#[doc(hidden)]` for internal APIs
- Generate documentation with `cargo doc`
- Include module-level documentation
- Add `# Examples` and `# Errors` sections
- Use intra-doc links for cross-references

```rust
//! # Graph Aggregate Module
//!
//! This module provides the core domain logic for graph operations.
//!
//! ## Example
//!
//! ```rust
//! use cim_graph::GraphAggregate;
//!
//! let mut graph = GraphAggregate::new("my-graph");
//! let event = graph.add_node(NodeType::Concept)?;
//! ```

/// Adds a new node to the graph.
///
/// # Arguments
///
/// * `node_type` - The type of node to add
///
/// # Returns
///
/// Returns a `NodeAdded` event on success.
///
/// # Errors
///
/// Returns `DomainError::InvalidState` if the graph is sealed.
///
/// # Example
///
/// ```
/// # use cim_graph::{GraphAggregate, NodeType};
/// # let mut graph = GraphAggregate::new("test");
/// let event = graph.add_node(NodeType::Concept)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn add_node(&mut self, node_type: NodeType) -> Result<NodeAdded, DomainError> {
    // Implementation
}
```

## Type System Usage

- Leverage Rust's type system for domain modeling
- Use newtypes for domain concepts
- Prefer enum variants over boolean flags
- Use phantom types for compile-time guarantees

```rust
// Newtype pattern for domain types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphId(Uuid);

impl GraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// State machines with phantom types
pub struct Graph<S> {
    id: GraphId,
    _state: PhantomData<S>,
}

pub struct Draft;
pub struct Published;

impl Graph<Draft> {
    pub fn publish(self) -> Graph<Published> {
        Graph {
            id: self.id,
            _state: PhantomData,
        }
    }
}

// Only published graphs can be queried
impl Graph<Published> {
    pub fn query(&self, q: Query) -> QueryResult {
        // Implementation
    }
}
```

## Async/Await Patterns

- Use `tokio` for async runtime
- Prefer `async-trait` for async traits
- Handle cancellation with `tokio::select!`
- Use `Arc<Mutex<T>>` sparingly, prefer message passing

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, events: Vec<Event>) -> Result<(), StoreError>;
    async fn load(&self, id: &str) -> Result<Vec<Event>, StoreError>;
}

// Graceful shutdown
pub async fn run_service(mut shutdown: Receiver<()>) {
    loop {
        tokio::select! {
            _ = process_next_item() => {
                // Continue processing
            }
            _ = shutdown.recv() => {
                info!("Shutting down gracefully");
                break;
            }
        }
    }
}
```

## Performance Guidelines

- Use `&str` instead of `String` for function parameters
- Prefer `Vec<T>` capacity hints with `with_capacity`
- Use `Cow<'a, str>` for potentially owned strings
- Profile before optimizing
- Consider `SmallVec` for small collections

```rust
// Efficient string handling
pub fn process_name<'a>(name: &'a str) -> Cow<'a, str> {
    if name.contains(' ') {
        Cow::Owned(name.replace(' ', "_"))
    } else {
        Cow::Borrowed(name)
    }
}

// Pre-allocate collections
pub fn collect_nodes(&self) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(self.estimated_size());
    // Fill nodes
    nodes
}
```

## Dependency Management

- Keep dependencies minimal
- Use `cargo-outdated` to track updates
- Pin versions for production
- Prefer well-maintained crates
- Audit dependencies with `cargo-audit`

```toml
[dependencies]
# Core dependencies only
tokio = { version = "1.43", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"

[dev-dependencies]
# Test dependencies
proptest = "1.6"
tokio-test = "0.4"
```

## Common Patterns

### Builder Pattern
```rust
#[derive(Default)]
pub struct GraphBuilder {
    name: Option<String>,
    config: Option<GraphConfig>,
}

impl GraphBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn build(self) -> Result<Graph, BuildError> {
        Ok(Graph {
            name: self.name.ok_or(BuildError::MissingName)?,
            config: self.config.unwrap_or_default(),
        })
    }
}
```

### Type State Pattern
```rust
pub struct Request<S> {
    data: RequestData,
    _state: PhantomData<S>,
}

pub struct Pending;
pub struct Validated;

impl Request<Pending> {
    pub fn validate(self) -> Result<Request<Validated>, ValidationError> {
        // Validation logic
        Ok(Request {
            data: self.data,
            _state: PhantomData,
        })
    }
}
```

## Code Examples

### Basic ECS System
```rust
fn health_system(
    mut commands: Commands,
    mut healths: Query<(Entity, &mut Health)>,
    time: Res<Time>,
) {
    for (entity, mut health) in &mut healths {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Domain Event Publishing
```rust
fn process_order(
    order: &Order,
    mut events: EventWriter<DomainEvent>,
) {
    // Process order logic
    events.send(DomainEvent::OrderProcessed {
        order_id: order.id,
        total: order.calculate_total(),
    });
}
```

### NATS Integration
```rust
async fn setup_nats() -> Result<Client, Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    
    // Subscribe to domain events
    let mut subscriber = client
        .subscribe("orders.>")
        .await?;
    
    Ok(client)
}
```