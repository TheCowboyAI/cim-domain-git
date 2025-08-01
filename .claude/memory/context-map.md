# Context Mapping for Progress Nodes

## Purpose
Maps progress.json nodes to specific code contexts and working directories.

## Node Type → Context Mapping

### Infrastructure Nodes
- **Pattern**: `nats-*`, `cluster-*`, `leaf-*`
- **Working Directory**: `/git/thecowboyai/cim/infrastructure/`
- **Context**: Infrastructure setup, NATS configuration
- **Key Files**: 
  - `*.conf` - NATS configuration files
  - `docker-compose.yml` - Container orchestration
  - `scripts/*.sh` - Setup and deployment scripts

### Client Implementation Nodes
- **Pattern**: `client-*`, `cim-client-*`
- **Working Directory**: `/git/thecowboyai/cim/client/`
- **Context**: Client-side NATS implementation
- **Key Files**:
  - `client.go` - Main client implementation
  - `connection.go` - Connection management
  - `handlers.go` - Message handlers

### Service Implementation Nodes
- **Pattern**: `service-*`, `leaf-service-*`
- **Working Directory**: `/git/thecowboyai/cim/services/`
- **Context**: NATS-enabled microservices
- **Key Files**:
  - `services/*/service.go` - Service implementations
  - `services/*/handlers.go` - Request handlers
  - `services/common/` - Shared service utilities

### Domain Module Nodes
- **Pattern**: `domain-*`, `module-*`
- **Working Directory**: `/git/thecowboyai/cim/domains/`
- **Context**: Business domain implementations
- **Key Files**:
  - `domains/*/aggregate.go` - Domain aggregates
  - `domains/*/events.go` - Domain events
  - `domains/*/commands.go` - Command handlers

## Context Switching Protocol

1. **Read Current Node Status**
   ```bash
   # Check active nodes in progress.json
   # Focus on IN_PROGRESS nodes
   ```

2. **Load Relevant Context**
   - Switch to appropriate working directory
   - Load context-specific instructions from `.claude/contexts/`
   - Review recent changes in that context

3. **Update Working Memory**
   ```md
   Current Context: [infrastructure|client|service|domain]
   Active Node: [node-id from progress.json]
   Dependencies: [list of dependent nodes]
   Next Actions: [based on node status]
   ```

## Integration with Todos

When starting work:
1. Check progress.json for active nodes
2. Create todos based on node requirements
3. Map todos to specific artifacts
4. Update progress.json after todo completion

Example:
```
Node: "nats-client-core" (IN_PROGRESS)
Todos:
- [ ] Implement connection manager
- [ ] Add reconnection logic
- [ ] Create request-reply handlers
- [ ] Write unit tests
```

## State Synchronization

**Before Context Switch:**
- Update progress.json with current state
- Commit any pending changes
- Document blockers if any

**After Context Switch:**
- Read new context from progress.json
- Load relevant memory files
- Check dependencies and blockers
- Plan work based on node status