# Sequential Thinking Template: NATS Service Creation

## Phase 1: Service Planning
1. Read progress.json for service context
2. Define service responsibilities
3. Design NATS subject hierarchy
4. Plan health check implementation

## Phase 2: Service Scaffold
```go
// Standard service structure
type Service struct {
    nc     *nats.Conn
    name   string
    health HealthChecker
}

// Required interfaces
type HealthChecker interface {
    Check(context.Context) error
}

type Handler interface {
    Handle(*nats.Msg)
}
```

## Phase 3: Implementation Steps
1. Connection setup
   - Connect to leaf node
   - Register service subjects
   - Implement graceful shutdown
2. Handler implementation
   - Request handlers
   - Event subscribers
   - Error responses
3. Monitoring
   - Health endpoint
   - Metrics collection
   - Logging setup

## Phase 4: Integration
1. Add to leaf node configuration
2. Update service discovery
3. Configure load balancing
4. Set up monitoring

## Phase 5: Documentation & Progress
1. Update progress.json
   - Add service node
   - Link dependencies
   - Update metrics
2. Document service API
3. Update architecture diagrams

## Quality Checklist
- [ ] Service follows CIM patterns
- [ ] Health checks implemented
- [ ] Graceful shutdown handled
- [ ] Tests cover failure modes
- [ ] Progress.json updated
- [ ] Artifacts documented