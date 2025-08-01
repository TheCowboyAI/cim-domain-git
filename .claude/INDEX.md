# .claude Directory Index

## Overview
This directory contains all instructions, patterns, and standards for Claude AI when working on the CIM (Composable Information Machine) project. The content is organized by context and priority.

## Directory Structure

### 📋 Primary Instructions
**Start here for core understanding**

- **[CLAUDE.md](./CLAUDE.md)** - Primary CIM instructions and critical rules
  - Context awareness (registry vs module)
  - Date handling requirements
  - Core architecture understanding
  - Development principles
  - Testing requirements

- **[CLAUDE_MODULE_TEMPLATE.md](./CLAUDE_MODULE_TEMPLATE.md)** - Instructions for use in modules
  - Context detection
  - Module-specific guidance
  - Integration patterns

- **[MODULE_SETUP.md](./MODULE_SETUP.md)** - How to add .claude to modules
  - Download instructions
  - Customization guide
  - Update procedures

### 📚 Instructions
**Detailed operational guidelines**

- **[instructions/main-directives.md](./instructions/main-directives.md)** - Main development directives
  - Critical filename conventions
  - Development environment setup
  - Quality standards
  - Progress tracking

- **[instructions/cim-core.md](./instructions/cim-core.md)** - CIM core implementation rules
  - Event sourcing requirements
  - NATS architecture patterns
  - Layer architecture
  - Implementation checklist

- **[instructions/cim-conversation-model.md](./instructions/cim-conversation-model.md)** - CIM conversation model ⭐
  - Living information paradigm
  - Event-driven architecture patterns
  - Semantic intelligence through geometry
  - Documentation requirements
  - Anti-patterns to avoid

- **[instructions/date-handling.md](./instructions/date-handling.md)** - Date handling rules
  - System date commands
  - Git date extraction
  - Common mistakes to avoid

- **[instructions/registry-management.md](./instructions/registry-management.md)** - CIM registry system
  - Module tracking and discovery
  - Event-driven updates
  - Query utilities
  - Private domain registry

### 🏗️ Architecture Patterns
**Design patterns and architectural guidelines**

- **[patterns/domain-driven-design.md](./patterns/domain-driven-design.md)** - DDD implementation
  - Zero CRUD violations
  - Value object patterns
  - Cross-domain integration

- **[patterns/ddd-ecs-isomorphic-mapping.md](./patterns/ddd-ecs-isomorphic-mapping.md)** - DDD-ECS mapping overview
  - Mathematical isomorphism
  - Component mappings
  - Structure preservation

- **[patterns/ddd-ecs-integration.md](./patterns/ddd-ecs-integration.md)** - DDD-ECS detailed integration ⭐
  - Complete isomorphic mapping details
  - Bounded context mapping
  - Aggregate consistency patterns
  - Event sourcing integration
  - Testing strategies

- **[patterns/event-sourcing.md](./patterns/event-sourcing.md)** - Event sourcing overview
  - CID chain requirements
  - NATS JetStream integration
  - Testing patterns

- **[patterns/event-sourcing-detailed.md](./patterns/event-sourcing-detailed.md)** - Detailed event sourcing ⭐
  - MANDATORY correlation/causation requirements
  - Persistable trait implementation
  - NATS message headers
  - Event stream validation
  - Common pitfalls and correct patterns

- **[patterns/conceptual-spaces.md](./patterns/conceptual-spaces.md)** - Semantic reasoning
  - Geometric knowledge representation
  - Distance metrics
  - AI integration

- **[patterns/graph-patterns.md](./patterns/graph-patterns.md)** - Graph architecture overview
  - EventStore design
  - CQRS implementation
  - Bevy ECS integration

- **[patterns/graph-mermaid-patterns.md](./patterns/graph-mermaid-patterns.md)** - Graph & Mermaid requirements ⭐
  - MANDATORY Mermaid diagram inclusion
  - Standard diagram types
  - Dog-fooding visualization
  - Performance patterns
  - Testing diagram requirements

- **[patterns/domain-assembly.md](./patterns/domain-assembly.md)** - **IMPORTANT: How to build CIMs**
  - Assembly-first approach
  - Module selection process
  - Domain extension patterns
  - Examples for different industries

- **[patterns/cim-registry.md](./patterns/cim-registry.md)** - Registry architecture
  - Repository conventions
  - Event-driven updates
  - Module discovery
  - Private domain tracking

### 📏 Technical Standards
**Language and tool-specific standards**

- **[standards/rust-coding-standards.md](./standards/rust-coding-standards.md)** - Rust conventions
  - ECS architecture guidelines
  - Error handling patterns
  - Performance considerations

- **[standards/rust-nix-integration.md](./standards/rust-nix-integration.md)** - Rust+Nix setup
  - Dynamic linking configuration
  - Dependency management
  - Common issues

- **[standards/nixos-development.md](./standards/nixos-development.md)** - NixOS development standards ⭐
  - Module creation patterns
  - Service configuration
  - Flakes integration
  - Testing strategies
  - Common patterns and anti-patterns

- **[standards/test-driven-development.md](./standards/test-driven-development.md)** - TDD requirements
  - Testing patterns
  - Mermaid documentation
  - Performance metrics

- **[standards/quality-assurance.md](./standards/quality-assurance.md)** - QA principles
  - Validation checklists
  - Documentation management
  - Quality metrics

- **[standards/mermaid-styling.md](./standards/mermaid-styling.md)** - Diagram styling
  - High-contrast colors
  - Consistent styling rules

- **[standards/production-readiness.md](./standards/production-readiness.md)** - **Production standards**
  - Criteria for production status
  - Current production modules (4)
  - Migration path for other modules
  - Review process

### 🧠 Memory System
**Project state and progress tracking**

- **[memory/README.md](./memory/README.md)** - Memory system overview
- **[memory/state.md](./memory/state.md)** - Current project state
- **[memory/update-protocol.md](./memory/update-protocol.md)** - How to update progress.json
- **[memory/context-map.md](./memory/context-map.md)** - Node to context mapping
- **[memory/git-state-tracking.md](./memory/git-state-tracking.md)** - Git hash integration

### 🎯 Context Switching
**NATS architecture contexts**

- **[contexts/client.md](./contexts/client.md)** - Client implementation context
- **[contexts/leaf.md](./contexts/leaf.md)** - Leaf node context
- **[contexts/cluster.md](./contexts/cluster.md)** - Cluster configuration
- **[contexts/super-cluster.md](./contexts/super-cluster.md)** - Global routing

### 📝 Templates
**Reusable implementation templates**

- **[templates/client-implementation.md](./templates/client-implementation.md)** - NATS client template
- **[templates/service-creation.md](./templates/service-creation.md)** - Service scaffold
- **[templates/infrastructure-setup.md](./templates/infrastructure-setup.md)** - Infrastructure template

### 🔄 Workflows
**Step-by-step implementation flows**

- **[workflows/new-cim-workflow.md](./workflows/new-cim-workflow.md)** - Creating new domain CIMs
- **[workflows/implementation-flow.md](./workflows/implementation-flow.md)** - Standard workflow

### 📜 Scripts
**Utility scripts for context and queries**

- **[scripts/detect-context.sh](./scripts/detect-context.sh)** - Detect module context
- **[scripts/query-modules.sh](../scripts/query-modules.sh)** - Query module registry
- **[scripts/query-graph.sh](../scripts/query-graph.sh)** - Query dependency graph

### 🏛️ Architecture Documentation
**System architecture references**

- **[architecture/module-dependencies.md](./architecture/module-dependencies.md)** - Module structure

### 🔒 Security
**Security configurations**

- **[security/settings.json](./security/settings.json)** - Permission settings

## Quick Start Guide

1. **First Time**: Read [CLAUDE.md](./CLAUDE.md) and [instructions/main-directives.md](./instructions/main-directives.md)
2. **Starting Work**: Check [memory/state.md](./memory/state.md) and `/doc/progress/progress.json`
3. **Implementation**: Follow relevant templates and patterns
4. **Context Switch**: Use appropriate context from [contexts/](./contexts/)
5. **Update Progress**: Follow [memory/update-protocol.md](./memory/update-protocol.md)

## Key Principles

1. **Assembly-First** - Build by assembling existing cim-* modules
2. **Single Domain Focus** - Each CIM serves one business domain
3. **Event-Driven Everything** - No CRUD, only events
4. **Graph-Based Workflows** - Visual programming paradigm
5. **NATS Architecture** - Distributed messaging backbone
6. **Progress Tracking** - Always update progress.json
7. **Git State** - Capture hashes for versioning

## Navigation by Task

### When creating a new domain CIM:
1. Clone `cim-start` template - **ALWAYS START HERE**
2. [workflows/new-cim-workflow.md](./workflows/new-cim-workflow.md) - Step-by-step process
3. [patterns/domain-assembly.md](./patterns/domain-assembly.md) - Assembly patterns
4. Review [/doc/cim_modules_catalog.md](/doc/cim_modules_catalog.md) - Available modules

### When implementing a new feature:
1. [instructions/cim-core.md](./instructions/cim-core.md) - Core rules
2. [patterns/event-sourcing.md](./patterns/event-sourcing.md) - Event patterns
3. [templates/](./templates/) - Implementation templates
4. [memory/update-protocol.md](./memory/update-protocol.md) - Update progress

### When working with NATS:
1. [contexts/](./contexts/) - Architecture contexts
2. [instructions/cim-core.md](./instructions/cim-core.md) - NATS patterns
3. [templates/client-implementation.md](./templates/client-implementation.md) - Client template

### When writing tests:
1. [standards/test-driven-development.md](./standards/test-driven-development.md) - TDD requirements
2. [patterns/event-sourcing.md](./patterns/event-sourcing.md) - Testing patterns
3. [standards/quality-assurance.md](./standards/quality-assurance.md) - QA standards

## Remember
- Always use system dates, never generate them
- Update progress.json after significant work
- Follow the layer architecture strictly
- Events are the single source of truth