# CIM Architecture Memory

## Core Components

### NATS Hierarchy
```
┌─────────────────────────────────────┐
│         Super-cluster               │
│  ┌─────────┬─────────┬─────────┐  │
│  │Cluster-1│Cluster-2│Cluster-3│  │
│  └────┬────┴────┬────┴────┬────┘  │
│       │         │         │        │
└───────┼─────────┼─────────┼────────┘
        │         │         │
   ┌────┴───┐┌────┴───┐┌────┴───┐
   │ Leaf-1 ││ Leaf-2 ││ Leaf-3 │
   └────┬───┘└────┬───┘└────┬───┘
        │         │         │
   ┌────┴───┐┌────┴───┐┌────┴───┐
   │Client-1││Client-2││Client-3│
   └────────┘└────────┘└────────┘
```

### Subject Naming Convention
- Client operations: `client.<id>.<action>`
- Service calls: `service.<name>.<method>`
- Health checks: `health.<component>.<check>`
- Metrics: `metrics.<component>.<metric>`
- Cluster ops: `cluster.<name>.<operation>`
- Global ops: `global.<region>.<operation>`

### Key Design Decisions
1. **Stateless Services**: Services maintain no local state
2. **JetStream for Persistence**: All persistent data in JetStream
3. **Subject-based Routing**: No hardcoded endpoints
4. **Health First**: Every component implements health checks
5. **Graceful Degradation**: Services continue with reduced functionality

### Performance Targets
- Client → Leaf latency: < 1ms
- Leaf → Cluster latency: < 5ms  
- Cross-cluster latency: < 50ms
- Cross-region latency: < 200ms
- Message throughput: 1M msgs/sec per node