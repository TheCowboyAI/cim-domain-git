# Sequential Thinking Template: NATS Client Implementation

## Phase 1: Analysis
1. Review progress.json for dependencies
2. Check existing client code patterns
3. Identify required NATS features
4. Plan connection architecture

## Phase 2: Core Implementation
1. Create connection manager
   - Connection pooling
   - Retry logic with backoff
   - Health monitoring
2. Implement message patterns
   - Request-Reply
   - Publish-Subscribe
   - Queue Groups
3. Error handling
   - Connection failures
   - Timeout handling
   - Circuit breakers

## Phase 3: Testing
1. Unit tests for each component
2. Integration tests with test NATS server
3. Failure scenario testing
4. Performance benchmarks

## Phase 4: Progress Update
1. Update progress.json with new node
2. Add artifacts list
3. Update completion percentage
4. Document in recent_changes

## Checklist
- [ ] Read current progress.json state
- [ ] Create implementation node in progress
- [ ] Implement core functionality
- [ ] Write comprehensive tests
- [ ] Update progress.json with completion
- [ ] Update next_priorities