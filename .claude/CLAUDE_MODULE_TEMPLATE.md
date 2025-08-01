# Claude Instructions for CIM Modules

## Context Detection

You are working in a CIM (Composable Information Machine) module. To understand your current context:

1. **Check Repository Name**: 
   - If `cim` → You're in the registry/source of truth
   - If `cim-*` → You're in a specific module
   - If `cim-domain-*` → You're in a domain implementation

2. **Check for Module Metadata**:
   - Look for `cim.yaml` in the root
   - Check `Cargo.toml` for module name and version
   - Review README.md for module purpose

## Module Context

### If in the Registry (`cim` repository)
You are in the **source of truth** for all CIM modules. This repository:
- Maintains the module registry and dependency graph
- Provides templates and standards
- Does NOT implement a CIM itself
- Acts as a passive assistant for CIM development

Your role here is to:
- Update registry when modules change
- Maintain documentation and standards
- Provide guidance on CIM development
- Track module dependencies and versions

### If in a Core Module (`cim-*`)
You are in a reusable module that provides specific functionality:
- Follow the single responsibility principle
- Maintain clean API boundaries
- Ensure compatibility with other modules
- Update version on breaking changes

### If in a Domain Module (`cim-domain-*`)
You are in a business domain implementation:
- This module assembles other CIM modules
- Implements domain-specific logic
- Should be thin - mostly configuration and wiring
- Focuses on ONE specific business domain

## Universal CIM Principles

Regardless of which module you're in:

### 1. Event-Driven Architecture
- **Everything is an event** - No CRUD operations
- Events are immutable and append-only
- Use CID chains for integrity
- Events flow through NATS

### 2. Module Assembly
- **Assemble, don't build** - Use existing modules
- Start with `cim-start` template for new CIMs
- Select appropriate modules from the registry
- Create thin domain extensions

### 3. NATS Communication
- All inter-module communication via NATS
- Subject naming: `module.entity.action`
- Request-reply for synchronous operations
- Publish-subscribe for events

### 4. Production Standards
Only 4 modules are production-ready:
- `cim-domain` - Base domain models and abstractions
- `cim-ipld` - Content addressing
- `cim-component` - ECS components
- `cim-subject` - Event routing

All others are in development.

## Working in This Module

### Initial Assessment
```bash
# Understand where you are
pwd
ls -la
cat cim.yaml 2>/dev/null || echo "No cim.yaml found"
grep "^name = " Cargo.toml 2>/dev/null || echo "No Cargo.toml found"

# Check production readiness
curl -s https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules-graph.json | \
  jq --arg mod "$(basename $PWD)" '.graph.nodes[$mod].status'
```

### Development Workflow

1. **Before Making Changes**:
   - Understand module's single responsibility
   - Check dependencies in registry
   - Review API compatibility

2. **Making Changes**:
   - Follow event-driven patterns
   - Maintain clean boundaries
   - Write tests first (TDD)
   - Document public APIs

3. **After Changes**:
   - Update version if breaking changes
   - Ensure CI passes
   - Module will auto-notify registry

### Module Integration

When integrating modules:
```rust
// In Cargo.toml
[dependencies]
cim-domain = { git = "https://github.com/thecowboyai/cim-domain" }
cim-security = { git = "https://github.com/thecowboyai/cim-security" }

// In your code
use cim_domain::{DomainEvent, Aggregate};
use cim_security::{SecurityContext, authorize};
```

### NATS Patterns

Standard NATS patterns for modules:
```rust
// Subscribe to module events
nc.subscribe("module.*.event")?;

// Publish module events
nc.publish("module.entity.created", &event)?;

// Request-reply for commands
let response = nc.request("module.command.execute", &command, timeout)?;
```

## Registry Integration

Every module should:
1. Include `.github/workflows/notify-cim-registry.yml`
2. Maintain `cim.yaml` with metadata
3. Use semantic versioning
4. Tag releases properly

## Quick Reference

### Check Module Status
```bash
# From any module
MODULE_NAME=$(basename $PWD)
curl -s https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules-graph.json | \
  jq --arg mod "$MODULE_NAME" '.graph.nodes[$mod]'
```

### Find Dependencies
```bash
# What this module depends on
curl -s https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules-graph.json | \
  jq --arg mod "$MODULE_NAME" '.graph.nodes[$mod].dependencies[]'
```

### Production Readiness
If this module is not production-ready, check:
- `.claude/standards/production-readiness.md` in the registry
- Work through the checklist systematically
- Aim for 80%+ test coverage
- Complete all documentation

## Context-Aware Assistance

Based on your current module:
- **In registry?** → Guide CIM development, don't implement
- **In core module?** → Focus on single responsibility
- **In domain module?** → Assemble and configure
- **In cim-start?** → This is a template, customize for domain

Remember: The registry (`thecowboyai/cim`) has the complete picture. When in doubt, check the registry for:
- Available modules
- Dependencies
- Production status
- Best practices