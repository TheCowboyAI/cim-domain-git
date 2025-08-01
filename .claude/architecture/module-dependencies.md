# CIM Module Architecture: Dependencies and Responsibilities

## Overview

This document provides a comprehensive view of all CIM (Composable Information Machine) modules, their single responsibilities, dependencies, and context boundaries. Each module follows the Single Responsibility Principle (SRP) and maintains clear bounded contexts as per Domain-Driven Design principles.

## Module Dependency Graph

```mermaid
graph TB
    %% Core Architecture Modules
    subgraph "Core Architecture Layer"
        cim-domain[cim-domain<br/>Domain Model Foundation]
        cim-component[cim-component<br/>Component System]
        cim-compose[cim-compose<br/>Composition Patterns]
        cim-subject[cim-subject<br/>Subject/Event Algebra]
        cim-bridge[cim-bridge<br/>Cross-Context Bridge]
    end

    %% Storage & Data Modules
    subgraph "Storage Layer"
        cim-ipld[cim-ipld<br/>IPLD Content Addressing]
        cim-ipld-graph[cim-ipld-graph<br/>IPLD Graph Storage]
        cim-flashstor[cim-flashstor<br/>Network Object Storage]
    end

    %% Graph Modules
    subgraph "Graph Systems Layer"
        cim-contextgraph[cim-contextgraph<br/>Context Boundaries]
        cim-conceptgraph[cim-conceptgraph<br/>Concept Relationships]
        cim-workflow-graph[cim-workflow-graph<br/>Workflow Execution]
    end

    %% Infrastructure
    subgraph "Infrastructure Layer"
        cim-infrastructure[cim-infrastructure<br/>Core Infrastructure]
        cim-network[cim-network<br/>Network Topologies]
        cim-keys[cim-keys<br/>Key Management]
        cim-security[cim-security<br/>Security Policies]
    end

    %% Domain Modules
    subgraph "Domain Layer"
        cim-domain-conceptualspaces[conceptualspaces<br/>Semantic Spaces]
        cim-domain-identity[identity<br/>Identity Management]
        cim-domain-bevy[bevy<br/>Bevy Integration]
        cim-domain-person[person<br/>Person Entities]
        cim-domain-organization[organization<br/>Org Entities]
        cim-domain-agent[agent<br/>AI Agents]
        cim-domain-policy[policy<br/>Business Rules]
        cim-domain-document[document<br/>Document Management]
        cim-domain-workflow[workflow<br/>Workflow Definitions]
        cim-domain-location[location<br/>Geographic Data]
        cim-domain-graph[graph<br/>Graph Operations]
        cim-domain-nix[nix<br/>Nix Integration]
        cim-domain-git[git<br/>Git Integration]
        cim-domain-dialog[dialog<br/>Conversation Management]
    end

    %% Agent Applications
    subgraph "Application Layer"
        cim-agent-alchemist[alchemist<br/>Main Application]
    end

    %% Dependencies
    cim-agent-alchemist --> cim-domain-conceptualspaces
    cim-agent-alchemist --> cim-domain-identity
    cim-agent-alchemist --> cim-domain-bevy
    cim-agent-alchemist --> cim-domain-person
    cim-agent-alchemist --> cim-domain-organization
    cim-agent-alchemist --> cim-domain-agent
    cim-agent-alchemist --> cim-domain-policy
    cim-agent-alchemist --> cim-domain-document
    cim-agent-alchemist --> cim-domain-workflow
    cim-agent-alchemist --> cim-domain-location
    cim-agent-alchemist --> cim-domain-graph
    cim-agent-alchemist --> cim-domain-nix
    cim-agent-alchemist --> cim-domain-git
    cim-agent-alchemist --> cim-domain-dialog

    cim-domain-conceptualspaces --> cim-domain
    cim-domain-identity --> cim-domain
    cim-domain-bevy --> cim-domain
    cim-domain-person --> cim-domain
    cim-domain-organization --> cim-domain
    cim-domain-agent --> cim-domain
    cim-domain-policy --> cim-domain
    cim-domain-document --> cim-domain
    cim-domain-workflow --> cim-domain
    cim-domain-location --> cim-domain
    cim-domain-graph --> cim-domain
    cim-domain-nix --> cim-domain
    cim-domain-git --> cim-domain
    cim-domain-dialog --> cim-domain

    cim-domain-graph --> cim-contextgraph
    cim-domain-graph --> cim-conceptgraph
    cim-domain-workflow --> cim-workflow-graph
    
    cim-contextgraph --> cim-ipld-graph
    cim-conceptgraph --> cim-ipld-graph
    cim-workflow-graph --> cim-ipld-graph
    
    cim-ipld-graph --> cim-ipld
    cim-flashstor --> cim-infrastructure
    
    cim-domain --> cim-bridge
    cim-bridge --> cim-subject
    cim-bridge --> cim-component
    cim-component --> cim-compose
    
    cim-domain --> cim-infrastructure
    cim-infrastructure --> cim-network
    cim-infrastructure --> cim-security
    cim-network --> cim-leaf
    cim-network --> cim-leaf-darwin
    cim-security --> cim-keys
    cim-security --> cim-domain-identity
    cim-security --> cim-domain-policy
```

## Layer Architecture

```mermaid
graph TD
    subgraph "Layered Architecture"
        APP[Application Layer<br/>User-facing applications]
        DOM[Domain Layer<br/>Business logic & entities]
        GRAPH[Graph Systems Layer<br/>Graph operations & workflows]
        CORE[Core Architecture Layer<br/>Foundational patterns]
        STORAGE[Storage Layer<br/>Persistence & content addressing]
        INFRA[Infrastructure Layer<br/>Cross-cutting concerns]
    end
    
    APP --> DOM
    DOM --> GRAPH
    DOM --> CORE
    GRAPH --> STORAGE
    CORE --> INFRA
    STORAGE --> INFRA
```

## Module Responsibilities

### Core Architecture Layer

#### cim-domain
- **Single Responsibility**: Provide base domain models and abstractions for all CIM domains
- **Context Boundary**: Core DDD patterns (Aggregates, Entities, Value Objects, Events)
- **Key Interfaces**: `Aggregate`, `DomainEvent`, `Command`, `Query`
- **Note**: This is the foundation that all domain modules build upon

#### cim-component
- **Single Responsibility**: Provide the component system for ECS architecture
- **Context Boundary**: Component definitions and component lifecycle management
- **Key Interfaces**: `Component`, `ComponentRegistry`, `ComponentQuery`

#### cim-compose
- **Single Responsibility**: Define composition patterns for combining components and behaviors
- **Context Boundary**: Composition rules and component combination strategies
- **Key Interfaces**: `Composable`, `CompositionRule`, `ComponentCombinator`

#### cim-subject
- **Single Responsibility**: Implement the subject/event algebra for message routing
- **Context Boundary**: Event correlation, causation tracking, and message identity
- **Key Interfaces**: `Subject`, `MessageIdentity`, `CorrelationId`, `CausationId`

#### cim-bridge
- **Single Responsibility**: Bridge between different architectural contexts (ECS ↔ DDD ↔ NATS)
- **Context Boundary**: Cross-context translation and adaptation patterns
- **Key Interfaces**: `ContextBridge`, `MessageAdapter`, `ComponentAdapter`

### Storage Layer

#### cim-ipld
- **Single Responsibility**: Provide IPLD (InterPlanetary Linked Data) content addressing
- **Context Boundary**: Content-addressed storage primitives and CID operations
- **Key Interfaces**: `Cid`, `IpldStore`, `ContentAddressable`

#### cim-ipld-graph
- **Single Responsibility**: Implement graph storage using IPLD for persistence
- **Context Boundary**: Graph persistence, traversal, and content-addressed graph operations
- **Key Interfaces**: `GraphStore`, `NodeStorage`, `EdgeStorage`

#### cim-flashstor
- **Single Responsibility**: Provide network-attached object storage for leaf nodes
- **Context Boundary**: Object storage operations, network attachment, and distributed storage
- **Key Interfaces**: `ObjectStore`, `NetworkStorage`, `LeafNodeConnector`

### Graph Systems Layer

#### cim-contextgraph
- **Single Responsibility**: Manage context boundaries and their relationships
- **Context Boundary**: Bounded context mapping and inter-context relationships
- **Key Interfaces**: `ContextGraph`, `ContextBoundary`, `ContextRelationship`

#### cim-conceptgraph
- **Single Responsibility**: Model conceptual relationships and semantic networks
- **Context Boundary**: Concept definitions, semantic relationships, and knowledge graphs
- **Key Interfaces**: `ConceptGraph`, `SemanticRelation`, `ConceptNode`

#### cim-workflow-graph
- **Single Responsibility**: Execute workflow definitions as directed graphs
- **Context Boundary**: Workflow execution engine and state management
- **Key Interfaces**: `WorkflowGraph`, `WorkflowNode`, `WorkflowExecutor`

### Infrastructure Layer

#### cim-infrastructure
- **Single Responsibility**: Provide core infrastructure services and utilities
- **Context Boundary**: Logging, metrics, configuration, and system utilities
- **Key Interfaces**: `Logger`, `MetricsCollector`, `ConfigProvider`

#### cim-network
- **Single Responsibility**: Manage network topologies, structures, and functionality
- **Context Boundary**: Network configuration, topology management, node discovery, and connectivity
- **Key Interfaces**: `NetworkTopology`, `NodeDiscovery`, `NetworkConfig`, `ConnectionManager`
- **Dependencies**: Built on cim-infrastructure for base services
- **Dependents**: Used by leaf nodes and edge computing modules

#### cim-keys
- **Single Responsibility**: Manage cryptographic keys and signatures
- **Context Boundary**: Key generation, storage, and cryptographic operations
- **Key Interfaces**: `KeyStore`, `Signer`, `Verifier`

#### cim-security
- **Single Responsibility**: Coordinate security across identity, policy, and key management
- **Context Boundary**: Unified security layer integrating authentication, authorization, and cryptography
- **Key Interfaces**: `SecurityCoordinator`, `AccessControl`, `SecurityContext`
- **Dependencies**: Works with cim-domain-identity, cim-domain-policy, and cim-keys to provide comprehensive security

### Domain Layer

#### cim-domain-conceptualspaces
- **Single Responsibility**: Implement conceptual spaces for semantic reasoning
- **Context Boundary**: Quality dimensions, conceptual regions, and similarity metrics
- **Key Interfaces**: `ConceptualSpace`, `QualityDimension`, `ConceptualPoint`

#### cim-domain-identity
- **Single Responsibility**: Manage identity for all entities in the system
- **Context Boundary**: Identity creation, validation, and lifecycle management
- **Key Interfaces**: `Identity`, `IdentityProvider`, `IdentityValidator`

#### cim-domain-bevy
- **Single Responsibility**: Integrate with Bevy ECS for visualization and interaction
- **Context Boundary**: Bevy-specific components, systems, and rendering
- **Key Interfaces**: `BevyComponent`, `BevySystem`, `RenderPipeline`

#### cim-domain-person
- **Single Responsibility**: Model person entities and their relationships
- **Context Boundary**: Person aggregate, personal information, and person-specific behaviors
- **Key Interfaces**: `PersonAggregate`, `PersonalInfo`, `PersonRelationship`

#### cim-domain-organization
- **Single Responsibility**: Model organizational structures and hierarchies
- **Context Boundary**: Organization aggregate, organizational units, and membership
- **Key Interfaces**: `OrganizationAggregate`, `OrgUnit`, `Membership`

#### cim-domain-agent
- **Single Responsibility**: Define AI agent capabilities and behaviors
- **Context Boundary**: Agent definitions, capabilities, and interaction patterns
- **Key Interfaces**: `Agent`, `AgentCapability`, `AgentBehavior`

#### cim-domain-policy
- **Single Responsibility**: Implement business rules and policy enforcement
- **Context Boundary**: Policy definitions, rule evaluation, and enforcement mechanisms
- **Key Interfaces**: `Policy`, `PolicyRule`, `PolicyEnforcer`

#### cim-domain-document
- **Single Responsibility**: Manage document lifecycle and content
- **Context Boundary**: Document storage, versioning, and metadata management
- **Key Interfaces**: `Document`, `DocumentVersion`, `DocumentMetadata`

#### cim-domain-workflow
- **Single Responsibility**: Define workflow templates and instances
- **Context Boundary**: Workflow definitions, state machines, and execution context
- **Key Interfaces**: `WorkflowDefinition`, `WorkflowInstance`, `WorkflowState`

#### cim-domain-location
- **Single Responsibility**: Handle geographic and spatial data
- **Context Boundary**: Geographic coordinates, regions, and spatial relationships
- **Key Interfaces**: `Location`, `GeographicRegion`, `SpatialRelation`

#### cim-domain-graph
- **Single Responsibility**: Provide graph operations and algorithms
- **Context Boundary**: Graph manipulation, traversal algorithms, and graph analytics
- **Key Interfaces**: `Graph`, `GraphAlgorithm`, `GraphQuery`

#### cim-domain-nix
- **Single Responsibility**: Integrate with Nix package manager and build system
- **Context Boundary**: Nix expressions, derivations, and build configurations
- **Key Interfaces**: `NixDerivation`, `NixBuilder`, `NixConfig`

#### cim-domain-git
- **Single Responsibility**: Integrate with Git version control system
- **Context Boundary**: Git operations, repository management, and version tracking
- **Key Interfaces**: `GitRepository`, `Commit`, `GitOperation`

#### cim-domain-dialog
- **Single Responsibility**: Manage conversational interactions and dialog state
- **Context Boundary**: Dialog management, conversation history, and interaction patterns
- **Key Interfaces**: `Dialog`, `ConversationState`, `DialogManager`

### Application Layer

#### cim-agent-alchemist
- **Single Responsibility**: Orchestrate all domains into a cohesive application
- **Context Boundary**: Application-level coordination, UI, and user interaction
- **Key Interfaces**: `Application`, `UserInterface`, `DomainOrchestrator`

## Context Boundaries

```mermaid
graph LR
    subgraph "Identity Context"
        ID[Identity Domain]
        PERSON[Person Domain]
        ORG[Organization Domain]
    end
    
    subgraph "Knowledge Context"
        CONCEPT[Conceptual Spaces]
        CONCEPTG[Concept Graph]
        DOC[Document Domain]
    end
    
    subgraph "Execution Context"
        WORKFLOW[Workflow Domain]
        WORKFLOWG[Workflow Graph]
        POLICY[Policy Domain]
    end
    
    subgraph "Interaction Context"
        AGENT[Agent Domain]
        DIALOG[Dialog Domain]
        BEVY[Bevy Domain]
    end
    
    subgraph "Technical Context"
        GIT[Git Domain]
        NIX[Nix Domain]
        GRAPH[Graph Domain]
    end
    
    ID --> PERSON
    ID --> ORG
    CONCEPT --> CONCEPTG
    WORKFLOW --> WORKFLOWG
    WORKFLOW --> POLICY
    AGENT --> DIALOG
```

## Communication Patterns

```mermaid
sequenceDiagram
    participant App as Application
    participant Domain as Domain Module
    participant Bridge as Context Bridge
    participant Core as Core Architecture
    participant Storage as Storage Layer
    
    App->>Domain: Command
    Domain->>Core: Domain Event
    Core->>Bridge: Translate Event
    Bridge->>Storage: Persist Event
    Storage-->>Bridge: CID
    Bridge-->>Core: Confirmation
    Core-->>Domain: Event Applied
    Domain-->>App: Result
```

## Design Principles

### 1. Single Responsibility Principle (SRP)
Each module has exactly one reason to change, encapsulating a single cohesive set of functionality.

### 2. Bounded Context Isolation
Modules communicate only through well-defined interfaces and events, never through direct dependencies on internal implementations.

### 3. Event-Driven Communication
All cross-context communication happens through events, maintaining loose coupling and enabling replay/audit capabilities.

### 4. Content Addressing
All persistent data uses content addressing (CIDs) for immutability and verifiability.

### 5. Layered Architecture
Dependencies flow downward through layers, with no upward or circular dependencies.

## Module Integration Guidelines

### Adding New Modules

1. **Identify the Layer**: Determine which architectural layer the module belongs to
2. **Define Single Responsibility**: Clearly articulate the one thing the module does
3. **Establish Context Boundary**: Define what is inside and outside the module's context
4. **Design Interfaces**: Create minimal, focused interfaces for interaction
5. **Plan Dependencies**: Only depend on modules in the same or lower layers

### Cross-Context Communication

1. **Use Events**: All cross-context communication must use domain events
2. **Bridge Translation**: Use the cim-bridge module for context translation
3. **No Direct Access**: Never access another context's internal state directly
4. **Async by Default**: Assume all cross-context operations are asynchronous

### Testing Strategy

1. **Unit Tests**: Test each module in isolation with mocked dependencies
2. **Integration Tests**: Test module interactions through their public interfaces
3. **Context Tests**: Verify context boundaries are maintained
4. **Event Tests**: Ensure events flow correctly between contexts

## Maintenance and Evolution

### Module Versioning
- Each module maintains its own semantic version
- Breaking changes require major version bumps
- Cross-module compatibility is managed through the flake.nix inputs

### Deprecation Process
1. Mark deprecated interfaces with warnings
2. Provide migration path in documentation
3. Support deprecated interfaces for at least one major version
4. Remove only after all dependent modules have migrated

### Documentation Requirements
- Each module must have a README explaining its single responsibility
- API documentation for all public interfaces
- Example usage for common scenarios
- Migration guides for breaking changes