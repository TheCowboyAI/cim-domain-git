# DDD-ECS Integration: Isomorphic Mapping

## Core Concept

The integration of Domain-Driven Design (DDD) with Entity Component System (ECS) creates an isomorphic mapping where:
- **Entities** map to **Aggregate Roots**
- **Components** map to **Value Objects** and **Domain Properties**
- **Systems** map to **Domain Services** and **Use Cases**
- **Resources** map to **Domain Repositories** and **Infrastructure Services**

## Isomorphic Mapping Details

### 1. Entity ↔ Aggregate Root

```rust
// DDD Aggregate Root
pub struct OrderAggregate {
    id: OrderId,
    customer_id: CustomerId,
    items: Vec<OrderItem>,
    status: OrderStatus,
    version: u64,
}

// ECS Entity with Components
#[derive(Component)]
struct OrderId(Uuid);

#[derive(Component)]
struct CustomerId(Uuid);

#[derive(Component)]
struct OrderItems(Vec<OrderItem>);

#[derive(Component)]
struct OrderStatus {
    state: OrderState,
    updated_at: SystemTime,
}

// Spawning an Order entity
fn spawn_order(mut commands: Commands, order: OrderAggregate) {
    commands.spawn((
        OrderId(order.id),
        CustomerId(order.customer_id),
        OrderItems(order.items),
        OrderStatus {
            state: order.status,
            updated_at: SystemTime::now(),
        },
    ));
}
```

### 2. Component ↔ Value Object

```rust
// DDD Value Object
#[derive(Debug, Clone, PartialEq)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub fn add(&self, other: &Money) -> Result<Money, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::CurrencyMismatch);
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}

// ECS Component (immutable by convention)
#[derive(Component, Clone)]
struct Price {
    money: Money,
}

// System respecting value object semantics
fn calculate_total_system(
    orders: Query<&OrderItems>,
    prices: Query<&Price>,
) -> Money {
    // Aggregate using value object methods
    orders.iter()
        .flat_map(|items| items.0.iter())
        .map(|item| prices.get(item.product_id).unwrap().money.clone())
        .fold(Money::zero(Currency::USD), |acc, price| {
            acc.add(&price).unwrap()
        })
}
```

### 3. System ↔ Domain Service

```rust
// DDD Domain Service
pub trait PricingService {
    fn calculate_discount(&self, order: &Order, customer: &Customer) -> Discount;
    fn apply_tax(&self, subtotal: Money, location: &Location) -> Money;
}

// ECS System implementing domain logic
fn pricing_system(
    mut orders: Query<(&OrderId, &OrderItems, &mut Price)>,
    customers: Query<(&CustomerId, &CustomerTier)>,
    tax_service: Res<TaxService>,
) {
    for (order_id, items, mut price) in &mut orders {
        // Calculate base price
        let subtotal = calculate_subtotal(&items);
        
        // Apply customer discount (domain logic)
        let customer_tier = customers.get(order_id.customer_id).unwrap();
        let discount = calculate_tier_discount(&customer_tier, &subtotal);
        
        // Apply tax (infrastructure service)
        let total = tax_service.calculate_total(subtotal - discount);
        
        price.money = total;
    }
}
```

### 4. Resource ↔ Repository

```rust
// DDD Repository trait
#[async_trait]
pub trait OrderRepository {
    async fn find_by_id(&self, id: OrderId) -> Result<Order, Error>;
    async fn save(&self, order: &Order) -> Result<(), Error>;
    async fn find_by_customer(&self, customer_id: CustomerId) -> Result<Vec<Order>, Error>;
}

// ECS Resource wrapping repository
#[derive(Resource)]
struct OrderStore {
    repository: Arc<dyn OrderRepository + Send + Sync>,
}

// System using repository through resource
fn persist_orders_system(
    changed_orders: Query<(&OrderId, &OrderStatus), Changed<OrderStatus>>,
    order_store: Res<OrderStore>,
    runtime: Res<TokioRuntime>,
) {
    for (order_id, status) in &changed_orders {
        let repository = order_store.repository.clone();
        let order_id = order_id.0;
        
        runtime.spawn(async move {
            // Load aggregate from repository
            let mut order = repository.find_by_id(order_id).await?;
            
            // Update aggregate
            order.update_status(status.state);
            
            // Persist through repository
            repository.save(&order).await?;
            
            Ok::<_, Error>(())
        });
    }
}
```

## Bounded Context Mapping

### Context Boundaries in ECS

```rust
// Each bounded context gets its own plugin
pub struct OrderManagementPlugin;
pub struct InventoryPlugin;
pub struct ShippingPlugin;

impl Plugin for OrderManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register components for this context
            .register_type::<OrderId>()
            .register_type::<OrderStatus>()
            .register_type::<OrderItems>()
            
            // Add systems for this context
            .add_systems(Update, (
                order_validation_system,
                order_pricing_system,
                order_persistence_system,
            ).chain())
            
            // Add resources for this context
            .insert_resource(OrderStore::new())
            .insert_resource(PricingRules::default());
    }
}

// Cross-context communication via events
#[derive(Event)]
struct OrderPlaced {
    order_id: OrderId,
    items: Vec<OrderItem>,
    total: Money,
}

// Inventory context listens to order events
fn inventory_reservation_system(
    mut order_events: EventReader<OrderPlaced>,
    mut inventory: ResMut<InventoryStore>,
) {
    for event in order_events.read() {
        // Reserve inventory based on order
        for item in &event.items {
            inventory.reserve(item.product_id, item.quantity);
        }
    }
}
```

## Aggregate Consistency

### Maintaining Invariants

```rust
// Bundle components to ensure aggregate consistency
#[derive(Bundle)]
struct OrderBundle {
    id: OrderId,
    customer: CustomerId,
    items: OrderItems,
    status: OrderStatus,
    total: Price,
    // Marker component for aggregate root
    #[bundle]
    aggregate: AggregateRoot,
}

#[derive(Bundle, Default)]
struct AggregateRoot {
    version: Version,
    last_modified: LastModified,
}

// System enforcing aggregate invariants
fn order_invariant_system(
    mut orders: Query<(&OrderItems, &OrderStatus, &mut Price), With<AggregateRoot>>,
) {
    for (items, status, mut price) in &mut orders {
        // Enforce: Cancelled orders have zero price
        if status.state == OrderState::Cancelled {
            price.money = Money::zero(price.money.currency);
        }
        
        // Enforce: Orders must have at least one item
        if items.0.is_empty() && status.state != OrderState::Draft {
            panic!("Invalid state: non-draft order with no items");
        }
    }
}
```

## Event Sourcing Integration

### Bridging Events and ECS

```rust
// Domain events as ECS events
#[derive(Event, Serialize, Deserialize)]
struct OrderEvent {
    aggregate_id: OrderId,
    sequence: u64,
    payload: OrderEventPayload,
    metadata: EventMetadata,
}

#[derive(Serialize, Deserialize)]
enum OrderEventPayload {
    Created { customer_id: CustomerId },
    ItemAdded { item: OrderItem },
    StatusChanged { old: OrderState, new: OrderState },
    Cancelled { reason: String },
}

// System that converts ECS state changes to domain events
fn event_sourcing_system(
    mut commands: Commands,
    changed_orders: Query<
        (Entity, &OrderId, &OrderStatus),
        Changed<OrderStatus>
    >,
    mut events: EventWriter<OrderEvent>,
    mut event_store: ResMut<EventStore>,
) {
    for (entity, order_id, status) in &changed_orders {
        let event = OrderEvent {
            aggregate_id: *order_id,
            sequence: get_next_sequence(&mut event_store, order_id),
            payload: OrderEventPayload::StatusChanged {
                old: get_previous_status(entity),
                new: status.state,
            },
            metadata: EventMetadata {
                timestamp: SystemTime::now(),
                correlation_id: get_correlation_id(),
                causation_id: get_causation_id(),
            },
        };
        
        // Emit to ECS event system
        events.send(event.clone());
        
        // Persist to event store
        event_store.append(event);
    }
}
```

## Query Model Projection

### CQRS Read Models

```rust
// Read model as ECS resource
#[derive(Resource, Default)]
struct OrderReadModel {
    orders_by_customer: HashMap<CustomerId, Vec<OrderSummary>>,
    orders_by_status: HashMap<OrderState, Vec<OrderId>>,
    daily_totals: HashMap<Date, Money>,
}

// System updating read model from events
fn projection_system(
    mut events: EventReader<OrderEvent>,
    mut read_model: ResMut<OrderReadModel>,
) {
    for event in events.read() {
        match &event.payload {
            OrderEventPayload::Created { customer_id } => {
                read_model.orders_by_customer
                    .entry(*customer_id)
                    .or_default()
                    .push(OrderSummary::from_event(event));
            }
            OrderEventPayload::StatusChanged { old, new } => {
                // Update status index
                if let Some(orders) = read_model.orders_by_status.get_mut(old) {
                    orders.retain(|id| id != &event.aggregate_id);
                }
                read_model.orders_by_status
                    .entry(*new)
                    .or_default()
                    .push(event.aggregate_id);
            }
            _ => {}
        }
    }
}

// Query system using read model
fn customer_orders_query(
    read_model: Res<OrderReadModel>,
    customer_id: CustomerId,
) -> Vec<OrderSummary> {
    read_model.orders_by_customer
        .get(&customer_id)
        .cloned()
        .unwrap_or_default()
}
```

## Testing Strategies

### Testing DDD Concepts in ECS

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    
    #[test]
    fn test_aggregate_invariants() {
        let mut world = World::new();
        
        // Create invalid aggregate state
        let order = world.spawn((
            OrderId(Uuid::new_v4()),
            OrderItems(vec![]), // Empty items
            OrderStatus { 
                state: OrderState::Confirmed, // Non-draft with no items
                updated_at: SystemTime::now(),
            },
        )).id();
        
        // System should panic on invariant violation
        let result = std::panic::catch_unwind(|| {
            world.run_system_once(order_invariant_system);
        });
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_domain_event_generation() {
        let mut world = World::new();
        world.init_resource::<Events<OrderEvent>>();
        
        // Create order that will change
        let order = world.spawn((
            OrderId(Uuid::new_v4()),
            OrderStatus {
                state: OrderState::Draft,
                updated_at: SystemTime::now(),
            },
        )).id();
        
        // Change status
        world.entity_mut(order)
            .get_mut::<OrderStatus>().unwrap()
            .state = OrderState::Confirmed;
        
        // Run event sourcing system
        world.run_system_once(event_sourcing_system);
        
        // Verify event was generated
        let events = world.resource::<Events<OrderEvent>>();
        assert_eq!(events.len(), 1);
    }
}
```

## Best Practices

### 1. Maintain Conceptual Integrity
- Keep domain concepts intact when mapping to ECS
- Use marker components to identify aggregate roots
- Bundle related components to maintain consistency

### 2. Respect Bounded Contexts
- Use plugins to encapsulate contexts
- Communicate between contexts via events only
- Don't share components across context boundaries

### 3. Handle Async Operations
- Use resources to wrap async repositories
- Convert between sync ECS and async domain services
- Use channels for complex async workflows

### 4. Preserve Domain Logic
- Keep business rules in systems, not components
- Use value objects for domain calculations
- Validate invariants consistently

### 5. Event-Driven Communication
- Use ECS events for immediate reactions
- Use domain events for audit and replay
- Maintain correlation across event chains

## Common Pitfalls

### ❌ Anemic Components
```rust
// WRONG - No behavior, just data
#[derive(Component)]
struct Price(f64);
```

### ✅ Rich Value Objects
```rust
// CORRECT - Encapsulated behavior
#[derive(Component)]
struct Price {
    money: Money, // Money has add(), subtract(), etc.
}
```

### ❌ Scattered Aggregate Logic
```rust
// WRONG - Aggregate logic spread across systems
fn system1() { /* partial validation */ }
fn system2() { /* more validation */ }
```

### ✅ Cohesive Aggregate Systems
```rust
// CORRECT - Single system maintains invariants
fn order_aggregate_system() {
    // All order invariants in one place
}
```

### ❌ Direct Cross-Context Access
```rust
// WRONG - Inventory directly modifies order
fn inventory_system(mut orders: Query<&mut OrderStatus>) {
    // Violates context boundaries
}
```

### ✅ Event-Based Integration
```rust
// CORRECT - Contexts communicate via events
fn inventory_system(mut events: EventReader<OrderPlaced>) {
    // React to order events
}
```

## Summary

The isomorphic mapping between DDD and ECS provides:
1. **Clear domain modeling** in a performance-oriented architecture
2. **Bounded context isolation** through ECS plugins
3. **Aggregate consistency** through component bundles
4. **Event sourcing** integration with ECS events
5. **CQRS support** through resources and systems

This approach maintains the benefits of both paradigms:
- DDD's focus on domain logic and business alignment
- ECS's performance and flexibility for complex simulations