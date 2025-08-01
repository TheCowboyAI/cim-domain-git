# Detailed Event Sourcing Patterns

## Event Persistence with Persistable Trait

Persistence is optional - implement the `Persistable` trait only where needed:

```rust
pub trait Persistable {
    fn persist(&self, store: &dyn EventStore) -> Result<Cid>;
}

// Only business-critical events implement Persistable
impl Persistable for OrderPlaced { /* ... */ }
impl Persistable for PaymentProcessed { /* ... */ }

// UI/technical events don't implement it
struct CacheInvalidated { /* ... */ } // No persistence
```

## MANDATORY: Event Correlation and Causation

### Core Requirements

Every event, command, and query in the system MUST include correlation and causation IDs:

```rust
pub struct MessageIdentity {
    pub message_id: MessageId,        // Unique for this message
    pub correlation_id: CorrelationId, // Groups related messages (REQUIRED)
    pub causation_id: CausationId,    // What caused this message (REQUIRED)
}

// Timestamp is separate metadata, NOT part of the correlation algebra
pub struct EventMetadata {
    pub identity: MessageIdentity,
    pub timestamp: SystemTime,
    pub actor: Option<ActorId>,
}
```

### Correlation/Causation Rules

1. **Root Messages**: `MessageId = CorrelationId = CausationId` (self-correlation)
2. **Caused Messages**: Inherit `CorrelationId` from parent, `CausationId = parent.MessageId`
3. **All NATS Messages**: Must include correlation headers (X-Correlation-ID, X-Causation-ID)
4. **Event Streams**: Must validate correlation chains and detect cycles

### Implementation Pattern

```rust
// ALWAYS use MessageFactory for creating messages
let root_cmd = MessageFactory::create_root(CreateOrder { ... });
let caused_event = MessageFactory::create_caused_by(OrderCreated { ... }, &root_cmd);

// NEVER create messages directly
let bad = Event { correlation_id: Uuid::new_v4(), ... }; // ❌ WRONG
```

## Event Structure with CID Chains

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    // Identity
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub sequence: u64,

    // CID Chain for integrity
    pub event_cid: Cid,
    pub previous_cid: Option<Cid>,

    // Payload
    pub event_type: String,
    pub payload: serde_json::Value,

    // Metadata
    pub timestamp: SystemTime,
    pub correlation_id: CorrelationId,  // REQUIRED, not optional
    pub causation_id: CausationId,      // REQUIRED, not optional
    pub actor: Option<ActorId>,
}
```

## NATS Message Headers

All NATS messages MUST include these headers:

```rust
impl NatsEventPublisher {
    pub async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        let headers = Headers::new()
            .add("X-Event-ID", event.event_id.to_string())
            .add("X-Correlation-ID", event.correlation_id.to_string())
            .add("X-Causation-ID", event.causation_id.to_string())
            .add("X-Aggregate-ID", event.aggregate_id.to_string())
            .add("X-Sequence", event.sequence.to_string())
            .add("X-Event-Type", &event.event_type);

        let subject = format!("events.{}.{}", 
            event.aggregate_type(), 
            event.aggregate_id
        );

        self.client
            .publish_with_headers(subject, headers, event.to_bytes()?)
            .await?;

        Ok(())
    }
}
```

## Event Store Integration

### NATS JetStream Configuration

```rust
// Stream per aggregate type
pub async fn create_event_stream(js: &jetstream::Context) -> Result<Stream> {
    js.create_stream(StreamConfig {
        name: "EVENTS".to_string(),
        subjects: vec![
            "events.graph.>".to_string(),
            "events.node.>".to_string(),
            "events.edge.>".to_string(),
        ],
        retention: RetentionPolicy::Limits,
        storage: StorageType::File,
        max_age: Duration::from_days(365),
        duplicate_window: Duration::from_secs(120),
        ..Default::default()
    }).await
}
```

### Event Publishing Pattern

```rust
impl EventStore {
    pub async fn append_events(
        &self,
        aggregate_id: AggregateId,
        events: Vec<DomainEvent>,
        expected_version: Option<u64>,
    ) -> Result<()> {
        // Optimistic concurrency control
        if let Some(version) = expected_version {
            self.verify_version(aggregate_id, version).await?;
        }

        // Calculate CID chain
        let mut previous_cid = self.get_latest_cid(aggregate_id).await?;

        for event in events {
            // Calculate event CID
            let event_cid = calculate_cid(&event, previous_cid)?;

            // Publish to NATS with required headers
            let headers = create_event_headers(&event);
            let subject = format!("events.{}.{}", 
                event.aggregate_type(), 
                aggregate_id
            );

            self.jetstream
                .publish_with_headers(subject, headers, event.to_bytes()?)
                .await?;

            previous_cid = Some(event_cid);
        }

        Ok(())
    }
}
```

## Event Stream Validation

```rust
pub struct EventStreamValidator {
    correlation_tracker: HashMap<CorrelationId, HashSet<MessageId>>,
}

impl EventStreamValidator {
    pub fn validate_event(&mut self, event: &DomainEvent) -> Result<()> {
        // Validate correlation chain
        self.validate_correlation_chain(event)?;
        
        // Detect cycles
        self.detect_causation_cycles(event)?;
        
        // Verify CID chain
        self.verify_cid_integrity(event)?;
        
        Ok(())
    }
    
    fn validate_correlation_chain(&mut self, event: &DomainEvent) -> Result<()> {
        let messages = self.correlation_tracker
            .entry(event.correlation_id.clone())
            .or_insert_with(HashSet::new);
            
        if !messages.insert(event.event_id.clone()) {
            return Err(Error::DuplicateMessage(event.event_id.clone()));
        }
        
        Ok(())
    }
}
```

## Aggregate Loading with Validation

```rust
pub async fn load_aggregate<A: Aggregate>(
    event_store: &EventStore,
    aggregate_id: AggregateId,
) -> Result<A> {
    let events = event_store
        .get_events(aggregate_id)
        .await?;

    let mut aggregate = A::default();
    let mut validator = EventStreamValidator::new();

    for event in events {
        // Validate event stream integrity
        validator.validate_event(&event)?;
        
        // Verify CID chain
        verify_cid_chain(&event, &aggregate.last_cid)?;

        // Apply event
        aggregate.apply_event(event)?;
    }

    Ok(aggregate)
}
```

## Snapshot Strategy

```rust
// Snapshot every N events or time period
pub struct SnapshotPolicy {
    pub event_threshold: u64,     // e.g., every 100 events
    pub time_threshold: Duration, // e.g., every hour
}

impl EventStore {
    pub async fn maybe_snapshot<A: Aggregate>(
        &self,
        aggregate: &A,
        policy: &SnapshotPolicy,
    ) -> Result<()> {
        if should_snapshot(aggregate, policy) {
            let snapshot = aggregate.to_snapshot()?;
            let snapshot_cid = self.object_store
                .put_object(snapshot)
                .await?;

            // Store with correlation metadata
            self.store_snapshot_reference(
                aggregate.id(),
                snapshot_cid,
                aggregate.version(),
                aggregate.last_correlation_id(),
            ).await?;
        }
        Ok(())
    }
}
```

## Event Versioning

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum NodeEvent {
    #[serde(rename = "1.0")]
    V1(NodeEventV1),

    #[serde(rename = "2.0")]
    V2(NodeEventV2),
}

// Handle multiple versions
impl From<NodeEvent> for DomainEvent {
    fn from(event: NodeEvent) -> Self {
        match event {
            NodeEvent::V1(e) => {
                // Convert old format, preserving correlation
                let v2 = upgrade_v1_to_v2(e);
                v2.into()
            },
            NodeEvent::V2(e) => e.into(),
        }
    }
}
```

## Idempotency with NATS

```rust
// Use event IDs for idempotency
impl EventStore {
    pub async fn append_event_idempotent(
        &self,
        event: DomainEvent,
    ) -> Result<()> {
        // NATS deduplication window handles this
        let headers = Headers::new()
            .add("Nats-Msg-Id", event.event_id.to_string())
            .add("X-Correlation-ID", event.correlation_id.to_string())
            .add("X-Causation-ID", event.causation_id.to_string());

        self.jetstream
            .publish_with_headers(subject, headers, payload)
            .await
    }
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("CID chain broken at sequence {0}")]
    BrokenCidChain(u64),

    #[error("Concurrent modification: expected version {expected}, got {actual}")]
    ConcurrentModification { expected: u64, actual: u64 },

    #[error("Event replay failed: {0}")]
    ReplayError(String),
    
    #[error("Invalid correlation chain: {0}")]
    InvalidCorrelation(String),
    
    #[error("Causation cycle detected: {0}")]
    CausationCycle(String),
    
    #[error("Missing required header: {0}")]
    MissingHeader(String),
}
```

## Common Pitfalls to Avoid

### ❌ Missing Correlation/Causation
```rust
// WRONG - Missing required IDs
let event = DomainEvent {
    correlation_id: None, // NOT ALLOWED
    causation_id: None,   // NOT ALLOWED
    ..
};
```

### ❌ Creating Messages Without Factory
```rust
// WRONG - Direct construction
let event = Event { 
    correlation_id: Uuid::new_v4(), 
    ..
};
```

### ❌ Mutable Events
```rust
// WRONG - Events must be immutable
event.timestamp = SystemTime::now();
```

### ❌ Business Logic in Event Handlers
```rust
// WRONG - Apply should only update state
fn apply_event(&mut self, event: Event) {
    if self.validate_business_rule() { // NO!
        self.state = event.new_state;
    }
}
```

## Correct Patterns

### ✅ Always Use MessageFactory
```rust
// CORRECT
let root = MessageFactory::create_root(command);
let event = MessageFactory::create_caused_by(domain_event, &root);
```

### ✅ Include All Headers
```rust
// CORRECT
let headers = Headers::new()
    .add("X-Correlation-ID", event.correlation_id.to_string())
    .add("X-Causation-ID", event.causation_id.to_string());
```

### ✅ Validate Event Streams
```rust
// CORRECT
validator.validate_correlation_chain(&event)?;
validator.verify_cid_integrity(&event)?;
```

### ✅ Immutable Event Storage
```rust
// CORRECT
struct NodeAdded {
    node_data: NodeSnapshot, // Data snapshot, not reference
    position: Position3D,
    components: Vec<ComponentData>,
}