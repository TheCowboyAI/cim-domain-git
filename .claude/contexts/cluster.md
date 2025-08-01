# Cluster Context

## Focus Area
NATS cluster configuration for high availability and scalability.

## Key Responsibilities
- Maintain cluster consensus
- Replicate JetStream data
- Handle node failures gracefully
- Balance client connections
- Coordinate service discovery

## Cluster Configuration
```conf
# cluster-node.conf
port: 4222

cluster {
  name: CIM_CLUSTER
  port: 6222
  
  routes = [
    nats://node1:6222
    nats://node2:6222
    nats://node3:6222
  ]
  
  # Cluster authorization
  authorization {
    user: cluster_route_user
    password: $CLUSTER_PASS
    timeout: 2
  }
}

# JetStream configuration
jetstream {
  store_dir: /data/jetstream
  max_memory_store: 4GB
  max_file_store: 100GB
}
```

## Failover Patterns
- Automatic client reconnection to available nodes
- JetStream raft consensus (R3 for critical data)
- Subject interest propagation
- Gateway connections to super-cluster