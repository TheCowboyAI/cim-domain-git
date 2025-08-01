# Domain-Driven Design Patterns

## Core Principles

### Zero CRUD Violations Rule

**MANDATORY**: All domains MUST follow event-driven architecture with NO CRUD violations.

### Value Object Immutability in Event Sourcing

Value Objects CANNOT be "updated" in Event Sourcing - they are replaced entirely.

```rust
// ❌ WRONG - Never create "update" events for value objects
pub enum EdgeEvent {
    EdgeUpdated {
        edge_id: EdgeId,
        old_relationship: EdgeRelationship,
        new_relationship: EdgeRelationship,
    }, // This violates DDD principles!
}

// ✅ CORRECT - Remove and recreate (PROVEN PATTERN)
pub enum EdgeEvent {
    EdgeRemoved { edge_id: EdgeId },
    EdgeAdded {
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
}
```

**Why This Matters**:
1. **Events are immutable facts** - They record what happened, not what changed
2. **Value Objects have no lifecycle** - They exist or don't exist, no in-between
3. **Clear event semantics** - Removal and addition are distinct business events
4. **Audit trail integrity** - Shows the complete replacement, not a partial mutation

### Proven Implementation Pattern

```rust
// When changing a value object property
impl GraphAggregate {
    pub fn change_edge_relationship(
        &mut self,
        edge_id: EdgeId,
        new_relationship: EdgeRelationship,
    ) -> Result<Vec<DomainEvent>> {
        let old_edge = self.edges.get(&edge_id)
            .ok_or(DomainError::EdgeNotFound)?;

        // Generate two events: removal then addition
        let events = vec![
            DomainEvent::EdgeRemoved {
                graph_id: self.id,
                edge_id,
            },
            DomainEvent::EdgeAdded {
                graph_id: self.id,
                edge_id: EdgeId::new(), // New identity
                source: old_edge.source,
                target: old_edge.target,
                relationship: new_relationship,
            },
        ];

        // Apply both events
        for event in &events {
            self.apply_event(event)?;
        }

        Ok(events)
    }
}
```

## Cross-Domain Integration Patterns

**PROVEN**: Git→Graph domain integration generates real workflows with 103+ events.

### Cross-Domain Event Flow
```rust
// Example: Git commits → Graph nodes
GitEvent::CommitAdded { commit_id, message, author } 
    → GraphCommand::AddNode { node_id: commit_id, content: message }
    → GraphEvent::NodeAdded { node_id, position, metadata }
```

### Integration Rules
1. **No Direct Dependencies**: Domains communicate only through events
2. **Event Translation**: Use converter/adapter patterns for cross-domain data
3. **Async Coordination**: Cross-domain workflows are eventually consistent
4. **Bounded Context Integrity**: Each domain maintains its own model

## Naming Conventions

### Ubiquitous Language
- All names must be derived from the business domain vocabulary
- Avoid technical terms and suffixes unless they are part of the domain language
- Collaborate with domain experts to validate names and maintain a shared glossary
- Names must be clear, pronounceable, and free from uninterpretable acronyms
- Compound Names should be natural phrases with no whitespace and PascalCase
- Keep names as concise and specific as possible

### Aggregates and Entities
- Aggregates are named as singular nouns (e.g., `Order`, `User`)
- Entities within aggregates are also named as singular nouns (e.g., `OrderItem`)
- Avoid technical suffixes unless required by the domain

### Domain Services
- Domain services are named as `ServiceContext` (e.g., `AuthorizeUserRegistration`, `ApproveInvoice`)
- Application services are named as `ServiceContext` (e.g., `UserRegistration`)
- Services should NOT reflect a hierarchy

### Repositories
- Repositories are named as `DomainContext` (e.g., `Orders`, `People`, `Organizations`)
- Avoid generic or ambiguous repository names

### Value Objects
- Value objects are named as descriptive nouns or noun phrases (e.g., `Address`, `TimeRange`)
- Value objects must be immutable and clearly distinguish themselves from entities

### Domain Events
- Domain events are preceded in Subject as `event.` so we should not repeat that pattern in the name
- Example: `MoneyDeposited`, `OrderPlaced`
- Events must be specific to the action and subject
- Event payloads should be minimal, immutable, and use primitive types or simple DTOs
- Event payloads may also be a CID, referring to an Object in the Object Store
- Event names must be serializable and independent of domain model classes

### Event-Driven Architecture (Topic/Queue Naming)
- Events related to collections or aggregates use plural names (e.g., `payments.failed`, `users.registered`)
- Events related to processes or single entities use singular names (e.g., `transaction.authorised`)
- IF Including a version, do so at the END of the topic name (e.g., `domain.event.v1`)
- For sub-entities or nested concepts, use plural for collections (e.g., `order.items.shipped`)
- Avoid embedding technical details or generic terms in event names

### Intention-Revealing Interfaces
- Interfaces and classes must reveal intent through their names
- Example: `CompleteInvoiceApproval`, not `InvoiceService`
- Avoid generic or ambiguous names
- Interfaces should be Atomic
- Interfaces may be Composed

### Bounded Contexts
- Concepts must be isolated within their bounded context
- Example: `Candidate` in "sourcing" context vs. `Prospect` in "interview" context
- Use context-specific names to avoid ambiguity

## Domain Completion Requirements

For a domain to be considered "complete" it must have:

1. **Event-Driven Architecture**: Zero CRUD violations, all operations through events
2. **Comprehensive Tests**: All handlers, aggregates, and queries tested
3. **CQRS Implementation**: Clear command/query separation with projections  
4. **Cross-Domain Integration**: Proven integration patterns with other domains
5. **Documentation**: Complete API documentation and usage examples

## Best Practices

### Naming Process and Documentation
- All naming conventions must be documented in a shared glossary
- Names must be reviewed and validated by both developers and domain experts
- Use collaborative modeling techniques (e.g., Event Storming) to refine names iteratively
- REFACTOR NAMES AS THE DOMAIN UNDERSTANDING EVOLVES

### Enforcement and Tools
- Use linters, style guides, or static analysis tools to enforce naming conventions
- DOCUMENT EXCEPTIONS AND RATIONALE FOR ANY DEVIATION FROM THE STANDARD
- Regularly audit code and documentation for compliance

**ADHERE STRICTLY TO THESE RULES TO MINIMIZE AMBIGUITY AND MAXIMIZE CLARITY**