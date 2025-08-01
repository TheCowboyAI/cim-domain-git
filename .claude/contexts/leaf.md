# Leaf Node Context

## Focus Area
Leaf node configuration, service orchestration, and routing.

## Network Foundation
Leaf nodes are built on `cim-network` which provides:
- Network topology management
- Node discovery mechanisms
- Connection management
- Network configuration

## Key Responsibilities
- Host multiple NATS-enabled services
- Route messages between client and cluster
- Load balance service requests
- Monitor service health
- Handle service discovery

## Configuration Template
```conf
# leaf.conf
port: 4222
cluster {
  port: 6222
  routes = [
    nats://cluster-node-1:6222
    nats://cluster-node-2:6222
    nats://cluster-node-3:6222
  ]
}

leafnodes {
  port: 7422
}

# Service accounts
accounts {
  $SYS { users = [ { user: admin, pass: $ADMIN_PASS } ] }
  
  SERVICE_A {
    jetstream: enabled
    users = [ { user: service_a, pass: $SERVICE_A_PASS } ]
  }
}
```

## Service Management
- Each service connects as a separate NATS client
- Services register their capabilities via subjects
- Health checks via `service.*.health` subjects
- Metrics collection via `service.*.metrics`