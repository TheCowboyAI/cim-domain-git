# Sequential Thinking Template: Infrastructure Setup

## Phase 1: Environment Analysis
1. Check progress.json infrastructure nodes
2. Review existing configuration
3. Identify deployment targets
4. Plan network topology

## Phase 2: NATS Configuration
1. Server configurations
   ```conf
   # Standard ports
   Client: 4222
   Cluster: 6222  
   Leafnode: 7422
   Gateway: 7222
   Monitoring: 8222
   ```
2. Security setup
   - TLS certificates
   - Authentication
   - Authorization
3. Clustering setup
   - Route definitions
   - Consensus settings

## Phase 3: Deployment Preparation
1. Container setup
   - Dockerfile creation
   - docker-compose.yml
   - Volume mounts
2. Configuration management
   - Environment variables
   - Secrets handling
   - Config templates

## Phase 4: Verification
1. Connectivity tests
2. Cluster formation verify
3. Security validation
4. Performance baseline

## Phase 5: Progress Documentation
1. Create/update infrastructure node
2. Document configuration artifacts
3. Update deployment guides
4. Record baseline metrics

## Artifact Checklist
- [ ] NATS server configs created
- [ ] Docker configurations ready
- [ ] Scripts for deployment
- [ ] Monitoring setup complete
- [ ] Progress.json updated with all artifacts