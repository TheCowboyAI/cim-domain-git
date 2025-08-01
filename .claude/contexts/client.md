# Client Context

## Focus Area
Local NATS client implementation and connection management.

## Key Responsibilities
- Establish secure connection to Leaf Node
- Handle connection resilience (reconnect logic)
- Implement request-reply patterns
- Subscribe to relevant subjects
- Local message queuing during disconnections

## Important Patterns
```go
// Connection with retry
nc, err := nats.Connect(leafURL, 
    nats.MaxReconnects(-1),
    nats.ReconnectWait(time.Second),
    nats.DisconnectErrHandler(handleDisconnect),
)

// Request with timeout
msg, err := nc.Request("service.action", data, 5*time.Second)
```

## Context Switching Triggers
- When implementing client-side features
- Debugging connection issues
- Optimizing client performance
- Implementing client-side caching