# CIM Development Assistant Instructions

## IMPORTANT: Context Awareness
**Check which repository you're in:**
- If in `cim` → You're in the REGISTRY (source of truth, not an implementation)
- If in `cim-*` → You're in a MODULE (provides specific functionality)
- If in `cim-domain-*` → You're in a DOMAIN (assembles modules for business)

Run `.claude/scripts/detect-context.sh` to understand your current context.

## CRITICAL: Date Handling Rules
**NEVER generate dates from memory. ALWAYS use:**
1. System date: `$(date -I)` or `$(date +%Y-%m-%d)`
2. Git commit dates: `$(git log -1 --format=%cd --date=short)`
3. Existing dates from files being read

**When updating progress.json:**
```bash
# Always capture system date
CURRENT_DATE=$(date -I)
# Then use $CURRENT_DATE in JSON updates
```

## Core Architecture Understanding

### What is a CIM?
CIM is an entire ecosystem with a specified purpose. For what ultimate "domain" are we constructing information for? Primarily this is your organization, a unit within an organization, or a project within a unit.

With sufficient hardware, this can all run on a single machine, but we do not recommend that as standard.

Standard:
- A client running a Nix development Environment operating as client
- A server running NixOS or Nix-Darwin operating as a Leaf Node

A **Client Infrastructure Module (CIM)** is a distributed system architecture where:
- A client runs NATS locally
- Communicates with a Leaf Node via NATS
- The Leaf Node hosts multiple NATS-enabled services
- Leaf Nodes can connect to:
  - A Cluster (3+ leaf nodes in a NATS cluster)
  - A Super-cluster (3+ clusters interconnected)

### CIM Development Approach
**We ASSEMBLE existing cim-* modules** rather than creating everything from scratch:
1. **Start with cim-start**: Clone this template to begin any new CIM
2. **Use Existing Modules**: Select from 37+ available cim-* modules
3. **Create Domain Extensions**: Map modules to specific business domains
4. **Single Purpose Focus**: Each CIM targets ONE specific domain:
   - Private Mortgage Lending (cim-domain-mortgage)
   - Manufacturing (cim-domain-manufacturing)
   - Retail (cim-domain-retail)
   - Healthcare (cim-domain-healthcare)
   - Or any other specific business domain

### CIM Registry
**This repository (thecowboyai/cim) is the source of truth**:
- Maintains registry of all official CIM modules
- Tracks private domain implementations
- Auto-updates when modules change
- Query via `./scripts/query-modules.sh`
- All `cim-*` repos in thecowboyai org are tracked

**IMPORTANT**: The registry does NOT implement a CIM itself. It:
- Provides standards, templates, and documentation
- Tracks all modules and their dependencies
- Acts as a passive assistant for CIM development
- Guides you through creating CIMs but doesn't create them

## Development Principles

### 1. NATS-First Architecture
- All service communication uses NATS messaging
- Prefer subject-based routing over direct connections
- Use request-reply patterns for synchronous operations
- Use publish-subscribe for event distribution

### 2. Service Design Patterns
- Each service should be stateless when possible
- Services must implement health checks via NATS
- Use NATS KV for configuration management
- Implement circuit breakers for resilience

### 3. Hierarchy and Scaling
```
Client (Local NATS)
  └── Leaf Node
      ├── Service A
      ├── Service B
      └── Service C
          └── Can connect to:
              ├── Cluster (3+ Leaf Nodes)
              └── Super-cluster (3+ Clusters)
```

## Code Organization

### Directory Structure
```
cim/
├── client/          # Client-side NATS implementation
├── leaf/            # Leaf node configuration and services
├── services/        # NATS-enabled microservices
├── cluster/         # Cluster configuration
├── super-cluster/   # Super-cluster configuration
└── .claude/         # Development assistant configuration
```

### Service Template
Every NATS service should follow this pattern:
1. Initialize NATS connection
2. Register service handlers
3. Implement health check responder
4. Handle graceful shutdown

## Context Switching Rules

When working on CIM components:
1. **Client Context**: Focus on local NATS setup, connection management
2. **Leaf Context**: Service orchestration, routing, load balancing
3. **Cluster Context**: High availability, data replication, failover
4. **Super-cluster Context**: Global routing, multi-region concerns

## Testing Requirements
- Unit tests for individual services
- Integration tests for NATS communication
- Cluster tests for failover scenarios
- Performance tests for message throughput

## Security Considerations
- TLS for all NATS connections
- JWT-based authentication
- Subject-based authorization
- Audit logging for all operations

## Critical Patterns and Standards

### Core Implementation Patterns
Refer to these essential patterns when implementing CIM features:

1. **[instructions/cim-conversation-model.md](./instructions/cim-conversation-model.md)** - Living information paradigm
   - Events as first-class citizens
   - No CRUD operations allowed
   - Semantic intelligence through geometry

2. **[patterns/event-sourcing-detailed.md](./patterns/event-sourcing-detailed.md)** - MANDATORY event patterns
   - All events MUST have correlation and causation IDs
   - Use Persistable trait for business-critical events
   - Follow NATS header requirements

3. **[patterns/graph-mermaid-patterns.md](./patterns/graph-mermaid-patterns.md)** - Visualization requirements
   - MANDATORY Mermaid diagrams in all graph modules
   - Dog-fooding: visualize CIM development itself
   - Standard diagram types for consistency

4. **[patterns/ddd-ecs-integration.md](./patterns/ddd-ecs-integration.md)** - Domain modeling
   - Isomorphic mapping between DDD and ECS
   - Bounded context enforcement
   - Aggregate consistency patterns

5. **[standards/nixos-development.md](./standards/nixos-development.md)** - NixOS standards
   - Module structure requirements
   - Service configuration patterns
   - Testing with nixos-rebuild