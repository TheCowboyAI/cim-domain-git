# Super-cluster Context

## Focus Area
Multi-cluster coordination and global routing.

## Key Responsibilities
- Connect multiple clusters via gateways
- Global subject routing
- Cross-region data replication
- Disaster recovery coordination
- Global service discovery

## Gateway Configuration
```conf
# gateway.conf
gateway {
  name: "Region-US-East"
  port: 7222
  
  gateways = [
    {name: "Region-US-West", url: "nats://west.cluster:7222"}
    {name: "Region-EU", url: "nats://eu.cluster:7222"}
    {name: "Region-APAC", url: "nats://apac.cluster:7222"}
  ]
  
  # Gateway authorization
  authorization {
    user: gateway_user
    password: $GATEWAY_PASS
  }
}
```

## Global Patterns
- Subject namespacing: `region.cluster.service.action`
- Cross-region JetStream mirroring
- Global KV stores for configuration
- Latency-aware routing
- Active-active or active-passive setups

## Considerations
- Network latency between regions
- Data sovereignty requirements
- Bandwidth costs for cross-region traffic
- Consensus delays in global operations