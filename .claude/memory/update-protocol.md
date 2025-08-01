# Progress.json Update Protocol

## CRITICAL: Update Frequency
You MUST update progress.json:
- After completing any significant task
- When starting a new implementation phase
- When discovering new dependencies
- After creating new artifacts (files/modules)

## Update Structure

### Adding New Nodes
```json
{
  "id": "unique-identifier",
  "type": "implementation|documentation|infrastructure|test",
  "status": "PLANNED|IN_PROGRESS|COMPLETE|BLOCKED",
  "description": "Clear description of what was done",
  "created": "YYYY-MM-DD",
  "completed": "YYYY-MM-DD",
  "git_hash": "abc123def456",  // Git commit hash when completed
  "metrics": {
    "relevant_metric": "value"
  },
  "artifacts": [
    "path/to/created/file.ext"
  ],
  "dependencies": ["node-id-1", "node-id-2"]
}
```

### Node Types for CIM
- `infrastructure`: NATS setup, cluster config, networking
- `implementation`: Service code, client code, business logic
- `documentation`: Specs, guides, architecture docs
- `test`: Test suites, integration tests, performance tests
- `configuration`: Config files, environment setup

### Status Transitions
- PLANNED → IN_PROGRESS → COMPLETE
- Any status → BLOCKED (with blocker info)
- BLOCKED → IN_PROGRESS (when unblocked)

### Adding Edges
```json
{
  "from": "source-node-id",
  "to": "target-node-id",
  "relationship": "depends-on|enables|implements|tests|documents"
}
```

### Updating Metrics
- Update `overall_completion` based on weighted progress
- Update domain-specific completion percentages
- Add specific metrics to track progress

### Recent Changes Entry
```json
{
  "date": "YYYY-MM-DD",
  "type": "implementation|configuration|documentation",
  "description": "What was accomplished",
  "impact": "LOW|MEDIUM|HIGH|CRITICAL",
  "details": [
    "Specific detail 1",
    "Specific detail 2"
  ]
}
```

## Example Update Flow

1. **Starting NATS Client Implementation**
```bash
# First capture the date
CURRENT_DATE=$(date -I)
```

```json
// Add node
{
  "id": "nats-client-core",
  "type": "implementation",
  "status": "IN_PROGRESS",
  "description": "Core NATS client with connection management",
  "created": "$CURRENT_DATE",
  "dependencies": ["cim-documentation"]
}
```

2. **After Completion**
```bash
# Capture both date and git hash
CURRENT_DATE=$(date -I)
GIT_HASH=$(git rev-parse HEAD)
```

```json
// Update node
{
  "status": "COMPLETE",
  "completed": "$CURRENT_DATE",
  "git_hash": "$GIT_HASH",
  "metrics": {
    "files_created": 3,
    "test_coverage": 85
  },
  "artifacts": [
    "client/nats_client.go",
    "client/connection_manager.go",
    "client/client_test.go"
  ]
}

// Add to recent_changes
{
  "date": "$CURRENT_DATE",
  "type": "implementation",
  "description": "Implemented core NATS client with reconnection logic",
  "impact": "HIGH",
  "details": [
    "Created connection manager with exponential backoff",
    "Implemented request-reply patterns",
    "Added comprehensive error handling"
  ]
}
```

## Remember
- ALWAYS update progress.json after significant work
- Use descriptive node IDs that reflect the component
- Track all created artifacts
- Update completion percentages accurately
- Maintain the graph structure with proper edges