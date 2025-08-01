# CIM Core Instructions

## What is CIM?

The **Composable Information Machine (CIM)** is a distributed system architecture built on:
- **Event-Driven Architecture**: All state changes through immutable events
- **Graph-Based Workflows**: Visual business processes and knowledge
- **NATS Messaging**: Distributed communication backbone
- **Conceptual Spaces**: Semantic relationship representation
- **Self-Referential Capability**: Systems that visualize themselves

## Module Assembly Approach

### We BUILD by ASSEMBLING, not creating from scratch

1. **Start with cim-start Template**
   ```bash
   # Clone the template for new CIM projects
   git clone <cim-start-repo> cim-<your-domain>
   ```

2. **Add Existing Modules**
   ```
   Core Modules Available:
   - cim-domain (base domain models - foundation for all domains)
   - cim-domain-identity (authentication)
   - cim-security (authorization)
   - cim-domain-policy (business rules)
   - cim-flashstor (object storage)
   - cim-domain-workflow (processes)
   - cim-network (network topologies and connectivity)
   - ... 38+ modules total
   ```

3. **Create Domain-Specific Extensions**
   ```
   Example: Private Mortgage Lending
   1. SELECT: cim-domain-identity, cim-domain-document, cim-domain-workflow
   2. EXTEND: Create cim-domain-mortgage with:
      - Loan aggregates
      - Underwriting workflows
      - Compliance policies
   3. CONFIGURE: Wire modules together for mortgage-specific needs
   ```

4. **Single Purpose Principle**
   - Each CIM implementation targets ONE specific business domain
   - Domain examples:
     - **cim-domain-mortgage**: Private lending workflows
     - **cim-domain-manufacturing**: Production line management
     - **cim-domain-retail**: Inventory and sales
     - **cim-domain-healthcare**: Patient care coordination

## Core Implementation Rules

### 1. Event Sourcing is Mandatory
```
(Command<T> | Query<T>) → [Events<T>] → Models/Projections
```
- **ZERO CRUD operations** - Only events modify state
- Every event is immutable with CID chains
- Events are the single source of truth
- Full replay capability to any point in time

### 2. NATS Architecture for CIM

#### Client-Leaf-Cluster Hierarchy
```
Client (Local NATS) → Leaf Node → Cluster → Super-cluster
```
- Clients run NATS locally
- Leaf nodes host NATS-enabled services
- Clusters provide HA (3+ leaf nodes)
- Super-clusters enable global distribution

#### Subject Naming Convention
- Client: `client.<id>.<action>`
- Service: `service.<name>.<method>`
- Health: `health.<component>.<check>`
- Events: `event.<aggregate>.<type>`
- Commands: `cmd.<aggregate>.<action>`

### 3. Layer Architecture
```
Presentation → Application → Domain → Infrastructure
     ↓             ↓            ↓           ↓
   Bevy UI    Handlers    Aggregates  NATS/Storage
```
**NEVER skip layers or expose infrastructure**

### 4. Command/Event Flow

1. **Command Creation**
   ```rust
   pub struct CreateNode {
       pub node_id: NodeId,
       pub node_type: NodeType,
       pub position: Position3D,
   }
   ```

2. **Aggregate Processing**
   ```rust
   impl NodeAggregate {
       pub fn handle_command(&mut self, cmd: Command) -> Result<Vec<DomainEvent>> {
           // Validate business rules
           // Generate events
           // Apply to self
       }
   }
   ```

3. **Event Publishing**
   ```rust
   // Events flow through NATS
   subject: "event.graph.node_created"
   payload: NodeCreated { ... }
   ```

### 5. Graph Types in CIM

1. **Workflow Graphs**: Business processes
2. **Conceptual Graphs**: Knowledge relationships
3. **Event Flow Graphs**: Event propagation
4. **Development Graphs**: Self-visualization

### 6. Dual ECS Pattern
```
Bevy ECS (Sync)              NATS Domain (Async)
├── Visual Components    ←→   ├── Domain Events
├── User Interactions    ←→   ├── Command Handlers
└── Real-time Updates    ←→   └── Event Streams
```

## Implementation Checklist

For every feature:
- [ ] Define domain events first
- [ ] Create commands for user intent
- [ ] Implement aggregate logic
- [ ] Set up NATS subjects
- [ ] Bridge async/sync properly
- [ ] Test each layer independently
- [ ] Update progress.json

## Anti-Patterns to Avoid

❌ **Direct state mutation**
```rust
// WRONG
node.position = new_position;
```

✅ **Event-driven update**
```rust
// CORRECT
commands.send(MoveNode { node_id, new_position });
```

❌ **Skipping layers**
```rust
// WRONG
bevy_system.nats_client.publish(...)
```

✅ **Proper abstraction**
```rust
// CORRECT
command_sender.send(Command)
```

## Key Differentiators

CIM is not just:
- Event Sourcing → Adds visual graphs and conceptual spaces
- Workflow Engine → Includes semantic understanding
- Graph Database → Combines with event streams
- Messaging System → Provides complete architecture

It's a **composable** system where each piece enhances the others.