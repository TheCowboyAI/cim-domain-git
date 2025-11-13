# cim-domain-git: Sprint Plan

**Version**: 1.0
**Date**: 2025-11-12
**Project**: git-IPLD Adapter Implementation
**Team Size**: 1 (AI-assisted)
**Sprint Duration**: Variable (tracked by tasks)

## Project Overview

Implement cim-domain-git as a mathematically rigorous adapter enabling cim-flashstor to store git repositories in native format while serving content through IPLD CID requests.

**Design Documents**:
- [DESIGN_CT.md](./DESIGN_CT.md) - Category Theory foundations
- [DESIGN_VERIFICATION.md](./DESIGN_VERIFICATION.md) - Mathematical verification

## Sprint Structure

Each sprint follows this workflow:
1. **Planning**: Define sprint goals and tasks
2. **Implementation**: Execute tasks
3. **Progress Tracking**: Update PROGRESS.md
4. **Testing**: Verify implementation
5. **Retrospective**: Document learnings
6. **Commit**: Git commit all changes

---

## Sprint 1: Foundation & Value Objects

**Duration**: 1 session
**Goal**: Implement core value objects and type system

### Sprint 1 Goals

1. ✅ Set up project structure with v0.8.1 standards
2. ✅ Implement all value object types with UUID v7
3. ✅ Create comprehensive test suite (100+ tests)
4. ✅ Ensure type safety and compile-time guarantees

### Sprint 1 Tasks

#### Task 1.1: Project Setup
- [ ] Create `cim-domain-git` repository structure
- [ ] Set up `Cargo.toml` with dependencies:
  - `uuid = { version = "1.11", features = ["v7", "serde"] }`
  - `serde = { version = "1.0", features = ["derive"] }`
  - `bytes = "1.5"`
  - `url = "2.5"`
  - `cid = "0.11"`
  - `multihash = "0.19"`
  - `async-trait = "0.1"`
  - `thiserror = "1.0"`
  - `chrono = "0.4"`
- [ ] Create `src/lib.rs` with module structure
- [ ] Create `.gitignore` for Rust projects
- [ ] Create `README.md` stub

#### Task 1.2: Core Value Objects
- [ ] Create `src/value_objects/mod.rs`
- [ ] Implement `GitHash` enum (SHA1, SHA256)
  - [ ] Serialization/deserialization
  - [ ] Display impl
  - [ ] 10+ tests
- [ ] Implement `GitRef` struct
  - [ ] repo_url, commit_hash, path fields
  - [ ] Validation
  - [ ] 10+ tests
- [ ] Implement `GitObjectType` enum
  - [ ] Commit, Tree, Blob, Tag variants
  - [ ] 5+ tests

#### Task 1.3: Git Object Types
- [ ] Implement `GitPerson` struct
  - [ ] name, email, timestamp, timezone
  - [ ] 10+ tests
- [ ] Implement `GitFileMode` enum
  - [ ] File, Executable, Symlink, Directory, Submodule
  - [ ] From/Into conversions for u32
  - [ ] 10+ tests
- [ ] Implement `GitTreeEntry` struct
  - [ ] mode, name, hash
  - [ ] 8+ tests

#### Task 1.4: Complex Git Objects
- [ ] Implement `GitCommit` struct
  - [ ] hash, tree, parents, author, committer, message, gpgsig
  - [ ] 15+ tests
- [ ] Implement `GitTree` struct
  - [ ] hash, entries
  - [ ] 12+ tests
- [ ] Implement `GitBlob` struct
  - [ ] hash, content
  - [ ] 10+ tests
- [ ] Implement `GitTag` struct
  - [ ] hash, object, object_type, tag_name, tagger, message, gpgsig
  - [ ] 12+ tests

#### Task 1.5: GitObject Sum Type
- [ ] Implement `GitObject` enum
  - [ ] Commit, Tree, Blob, Tag variants
  - [ ] Pattern matching helpers
  - [ ] 10+ tests

#### Task 1.6: Error Types
- [ ] Implement `GitIpldError` enum
  - [ ] InvalidHash, ObjectNotFound, NotGitContent, FetchError, IpldEncodingError
  - [ ] Error display
  - [ ] 8+ tests

### Sprint 1 Deliverables

- [ ] `src/value_objects/` module complete
- [ ] All value objects with comprehensive tests
- [ ] Documentation for each type
- [ ] Compilation: 0 errors, 0 warnings
- [ ] Tests: 100+ passing

### Sprint 1 Acceptance Criteria

1. All value object types compile without warnings
2. Test coverage >90% for value objects
3. All types implement required traits (Debug, Clone, PartialEq, Serialize, Deserialize)
4. GitHash supports both SHA-1 and SHA-256
5. Documentation complete for public API

---

## Sprint 2: Functors & IPLD Mapping

**Duration**: 1 session
**Goal**: Implement Category Theory functors F: Git → IPLD and G: IPLD → Git

### Sprint 2 Goals

1. ✅ Implement IpldNode types for dag-git encoding
2. ✅ Implement Functor trait and instances
3. ✅ Implement bidirectional mapping (fmap_to_ipld, fmap_from_ipld)
4. ✅ Verify functor laws hold
5. ✅ Test round-trip conversions

### Sprint 2 Tasks

#### Task 2.1: IPLD Node Types
- [ ] Create `src/ipld/mod.rs`
- [ ] Implement `IpldNode` enum
  - [ ] DagGitCommit variant
  - [ ] DagGitTree variant
  - [ ] DagGitBlob variant
  - [ ] DagGitTag variant
- [ ] Implement `IpldTreeEntry` struct
- [ ] Add CID generation helpers
- [ ] 15+ tests for IPLD types

#### Task 2.2: Functor Trait
- [ ] Create `src/functor.rs`
- [ ] Define `Functor<A, B>` trait
  - [ ] fn fmap(&self, a: A) -> B
- [ ] 5+ tests for trait

#### Task 2.3: GitToIpld Functor
- [ ] Implement `GitToIpld` struct
- [ ] Implement `Functor<GitObject, IpldNode> for GitToIpld`
  - [ ] fmap_commit implementation
  - [ ] fmap_tree implementation
  - [ ] fmap_blob implementation
  - [ ] fmap_tag implementation
- [ ] Helper: `git_hash_to_cid(hash, obj_type) -> Cid`
- [ ] 20+ tests

#### Task 2.4: IpldToGit Functor
- [ ] Implement `IpldToGit` struct
- [ ] Implement `Functor<IpldNode, Option<GitObject>> for IpldToGit`
  - [ ] fmap_commit implementation (inverse)
  - [ ] fmap_tree implementation (inverse)
  - [ ] fmap_blob implementation (inverse)
  - [ ] fmap_tag implementation (inverse)
- [ ] Helper: `cid_to_git_hash(cid) -> GitHash`
- [ ] 20+ tests

#### Task 2.5: GitIpldAdapter
- [ ] Create `src/adapter.rs`
- [ ] Implement `GitIpldAdapter` struct
  - [ ] to_ipld() method
  - [ ] from_ipld() method
- [ ] 15+ tests

#### Task 2.6: Functor Law Verification
- [ ] Property test: F(id) = id
- [ ] Property test: F(g ∘ f) = F(g) ∘ F(f)
- [ ] Property test: G(id) = id
- [ ] Property test: G(g ∘ f) = G(g) ∘ G(f)
- [ ] Property test: G(F(git_obj)) ≈ git_obj
- [ ] Property test: F(G(ipld_node)) ≈ ipld_node
- [ ] 10+ property tests (use proptest crate)

### Sprint 2 Deliverables

- [ ] `src/ipld/` module complete
- [ ] `src/functor.rs` complete
- [ ] `src/adapter.rs` complete
- [ ] All functor laws verified
- [ ] Round-trip tests passing
- [ ] Tests: 85+ new tests (185+ total)

### Sprint 2 Acceptance Criteria

1. GitToIpld functor correctly converts all git object types
2. IpldToGit functor correctly inverts conversions
3. Round-trip: Git → IPLD → Git preserves data
4. Round-trip: IPLD → Git → IPLD preserves CIDs
5. Property tests verify functor laws
6. Compilation: 0 errors, 0 warnings

---

## Sprint 3: Events, Commands & Domain Model

**Duration**: 1 session
**Goal**: Implement event sourcing and CQRS patterns

### Sprint 3 Goals

1. ✅ Define domain events for git operations
2. ✅ Define commands for git-IPLD operations
3. ✅ Implement event correlation and causation
4. ✅ Create aggregate for GitRepository
5. ✅ Test event application and replay

### Sprint 3 Tasks

#### Task 3.1: Domain Events
- [ ] Create `src/events/mod.rs`
- [ ] Implement `RepositoryRegistered` event
  - [ ] repo_url, refs, timestamp, correlation_id
  - [ ] 10+ tests
- [ ] Implement `ContentFetched` event
  - [ ] git_ref, cid, object_type, timestamp, correlation_id, causation_id
  - [ ] 12+ tests
- [ ] Implement `ObjectCached` event
  - [ ] cid, git_object, timestamp, correlation_id, causation_id
  - [ ] 12+ tests
- [ ] Implement `CidMappingCreated` event
  - [ ] cid, git_ref, timestamp, correlation_id, causation_id
  - [ ] 12+ tests
- [ ] Implement `GitDomainEvent` enum
  - [ ] Wraps all events
  - [ ] Helper methods
  - [ ] 10+ tests

#### Task 3.2: Commands
- [ ] Create `src/commands/mod.rs`
- [ ] Implement `RegisterRepository` command
  - [ ] repo_url, correlation_id
  - [ ] Validation
  - [ ] 10+ tests
- [ ] Implement `FetchContent` command
  - [ ] git_ref, correlation_id
  - [ ] Validation
  - [ ] 10+ tests
- [ ] Implement `ResolveCid` command
  - [ ] cid, correlation_id
  - [ ] Validation
  - [ ] 8+ tests
- [ ] Implement `CacheObject` command
  - [ ] git_object, correlation_id
  - [ ] Validation
  - [ ] 10+ tests
- [ ] Implement `GitDomainCommand` enum
  - [ ] Wraps all commands
  - [ ] Validation
  - [ ] 10+ tests

#### Task 3.3: GitRepository Aggregate
- [ ] Create `src/aggregate/mod.rs`
- [ ] Implement `GitRepository` struct
  - [ ] url, registered_at, refs, cid_mappings, version
  - [ ] new() constructor
  - [ ] apply_event() method (immutable)
  - [ ] replay_events() method
  - [ ] 25+ tests

### Sprint 3 Deliverables

- [ ] `src/events/` module complete (4 events + enum)
- [ ] `src/commands/` module complete (4 commands + enum)
- [ ] `src/aggregate/` module complete
- [ ] Event sourcing working
- [ ] Tests: 119+ new tests (304+ total)

### Sprint 3 Acceptance Criteria

1. All events include correlation/causation IDs
2. All commands validate inputs
3. GitRepository aggregate applies events immutably
4. Event replay reconstructs aggregate state
5. Compilation: 0 errors, 0 warnings

---

## Sprint 4: Infrastructure & Repository

**Duration**: 1 session
**Goal**: Implement event store, snapshot store, and repository pattern

### Sprint 4 Goals

1. ✅ Implement EventStore trait and in-memory implementation
2. ✅ Implement SnapshotStore trait and in-memory implementation
3. ✅ Implement Repository pattern with snapshot optimization
4. ✅ Implement git fetching (libgit2 or git2-rs)
5. ✅ Test persistence and replay

### Sprint 4 Tasks

#### Task 4.1: EventStore
- [ ] Create `src/infrastructure/event_store.rs`
- [ ] Define `EventStore` trait
  - [ ] append_events()
  - [ ] load_events()
  - [ ] load_events_after()
- [ ] Implement `InMemoryEventStore`
  - [ ] HashMap<GitRepoUrl, Vec<GitDomainEvent>>
  - [ ] Optimistic concurrency control
  - [ ] 15+ tests

#### Task 4.2: SnapshotStore
- [ ] Create `src/infrastructure/snapshot_store.rs`
- [ ] Define `GitRepositorySnapshot` struct
  - [ ] repo_url, version, aggregate, timestamp
- [ ] Define `SnapshotStore` trait
  - [ ] save_snapshot()
  - [ ] load_snapshot()
- [ ] Implement `InMemorySnapshotStore`
  - [ ] HashMap<GitRepoUrl, GitRepositorySnapshot>
  - [ ] 10+ tests

#### Task 4.3: Repository Pattern
- [ ] Create `src/infrastructure/repository.rs`
- [ ] Implement `GitRepositoryRepository` struct
  - [ ] event_store, snapshot_store, snapshot_frequency
  - [ ] new()
  - [ ] load() - with snapshot optimization
  - [ ] save() - with snapshot creation
  - [ ] 20+ tests

#### Task 4.4: Git Fetching
- [ ] Add dependency: `git2 = "0.18"` (libgit2 bindings)
- [ ] Create `src/git_fetch.rs`
- [ ] Implement `fetch_git_object(git_ref) -> Result<GitObject>`
  - [ ] Clone/open repo
  - [ ] Find commit
  - [ ] Traverse to object at path
  - [ ] Convert to GitObject
  - [ ] 15+ tests (integration tests)

### Sprint 4 Deliverables

- [ ] `src/infrastructure/` module complete
- [ ] `src/git_fetch.rs` complete
- [ ] Event sourcing infrastructure working
- [ ] Git object fetching working
- [ ] Tests: 60+ new tests (364+ total)

### Sprint 4 Acceptance Criteria

1. EventStore persists and retrieves events correctly
2. SnapshotStore optimizes event replay
3. Repository saves and loads aggregates with snapshots
4. Git fetching retrieves objects from remote repos
5. Integration test: fetch → convert → store → retrieve
6. Compilation: 0 errors, 0 warnings

---

## Sprint 5: NATS Integration & Service Binary

**Duration**: 1 session
**Goal**: Implement NATS integration and production service binary

### Sprint 5 Goals

1. ✅ Define NATS subject patterns for git domain
2. ✅ Implement NATS event publisher
3. ✅ Implement NATS command subscriber
4. ✅ Create `git-service` binary
5. ✅ Test end-to-end workflow

### Sprint 5 Tasks

#### Task 5.1: NATS Subjects
- [ ] Create `src/nats/subjects.rs`
- [ ] Implement `GitSubjects` struct
  - [ ] command(command_type) -> String
  - [ ] event(repo_url, event_type) -> String
  - [ ] repo_events(repo_url) -> String
  - [ ] all_events() -> String
  - [ ] 10+ tests

#### Task 5.2: NATS Event Publisher
- [ ] Create `src/nats/event_publisher.rs`
- [ ] Add dependency: `async-nats = "0.44"`
- [ ] Implement `NatsEventPublisher`
  - [ ] new(client)
  - [ ] publish_event()
  - [ ] publish_events()
  - [ ] 10+ tests

#### Task 5.3: Command Handlers
- [ ] Create `src/handlers/mod.rs`
- [ ] Implement `handle_register_repository()`
  - [ ] Deserialize command
  - [ ] Validate
  - [ ] Create event
  - [ ] Save to repository
  - [ ] Publish event
  - [ ] 10+ tests
- [ ] Implement `handle_fetch_content()`
  - [ ] Fetch git object
  - [ ] Convert to IPLD
  - [ ] Generate CID
  - [ ] Create events
  - [ ] Save and publish
  - [ ] 15+ tests
- [ ] Implement `handle_resolve_cid()`
  - [ ] Look up CID mapping
  - [ ] Return GitRef
  - [ ] 8+ tests

#### Task 5.4: git-service Binary
- [ ] Create `src/bin/git-service.rs`
- [ ] Implement main() with:
  - [ ] NATS connection
  - [ ] Infrastructure setup (event store, snapshot store, repository)
  - [ ] NATS subscriptions to all commands
  - [ ] Command routing to handlers
  - [ ] Graceful shutdown (Ctrl+C)
  - [ ] Structured logging (tracing)
- [ ] Add to Cargo.toml: `[[bin]]` section
- [ ] Integration test: full workflow

### Sprint 5 Deliverables

- [ ] `src/nats/` module complete
- [ ] `src/handlers/` module complete
- [ ] `src/bin/git-service.rs` complete
- [ ] Production binary builds (debug + release)
- [ ] Tests: 53+ new tests (417+ total)

### Sprint 5 Acceptance Criteria

1. NATS subjects follow `git.commands.*` and `git.events.*` patterns
2. Event publisher sends events to correct subjects
3. Command handlers process commands and publish events
4. git-service binary runs without errors
5. End-to-end test: Command → Handler → Event → Store
6. Binary size <15 MB (release)
7. Compilation: 0 errors, 0 warnings

---

## Sprint 6: Documentation & Final Polish

**Duration**: 1 session
**Goal**: Create comprehensive documentation and prepare for release

### Sprint 6 Goals

1. ✅ Write README.md with complete usage guide
2. ✅ Write USAGE.md with API examples
3. ✅ Write DEPLOYMENT.md with deployment strategies
4. ✅ Update all code documentation
5. ✅ Final testing and bug fixes

### Sprint 6 Tasks

#### Task 6.1: README.md
- [ ] Write comprehensive README (~4,000 words)
  - [ ] Overview and architecture
  - [ ] Category Theory explanation
  - [ ] Quick start guide
  - [ ] Installation instructions
  - [ ] Basic usage examples
  - [ ] API documentation
  - [ ] Testing guide
  - [ ] License and contributing

#### Task 6.2: USAGE.md
- [ ] Write detailed usage guide (~5,000 words)
  - [ ] Value object usage
  - [ ] Functor examples
  - [ ] Command/event patterns
  - [ ] Repository usage
  - [ ] NATS integration examples
  - [ ] git-service deployment
  - [ ] Troubleshooting

#### Task 6.3: DEPLOYMENT.md
- [ ] Write deployment guide (~6,000 words)
  - [ ] Docker deployment
  - [ ] Kubernetes deployment
  - [ ] Systemd service
  - [ ] NATS configuration
  - [ ] Monitoring and observability
  - [ ] Security considerations
  - [ ] Backup and recovery

#### Task 6.4: Code Documentation
- [ ] Add rustdoc comments to all public APIs
- [ ] Add module-level documentation
- [ ] Add examples in doc comments
- [ ] Generate docs: `cargo doc --no-deps --open`
- [ ] Fix any doc warnings

#### Task 6.5: Final Testing
- [ ] Run full test suite: `cargo test`
- [ ] Run with coverage: `cargo tarpaulin`
- [ ] Fix any failing tests
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Run fmt: `cargo fmt -- --check`
- [ ] Integration test suite

#### Task 6.6: Performance Testing
- [ ] Benchmark git object conversion (criterion)
- [ ] Benchmark CID generation
- [ ] Benchmark event replay (1000+ events)
- [ ] Profile memory usage
- [ ] Document performance characteristics

### Sprint 6 Deliverables

- [ ] README.md (~4,000 words)
- [ ] USAGE.md (~5,000 words)
- [ ] DEPLOYMENT.md (~6,000 words)
- [ ] Complete rustdoc documentation
- [ ] All tests passing (417+ tests)
- [ ] Benchmarks complete
- [ ] Code quality: clippy, fmt clean

### Sprint 6 Acceptance Criteria

1. Documentation complete and high-quality
2. All public APIs documented with examples
3. Test coverage >85%
4. No clippy warnings
5. Code formatted consistently
6. Performance benchmarks documented
7. Ready for v0.8.1 release

---

## Overall Project Metrics

### Code Targets

| Metric | Target | Sprint Achieved |
|--------|--------|-----------------|
| Value Objects | 10+ types | Sprint 1 |
| Events | 4+ types | Sprint 3 |
| Commands | 4+ types | Sprint 3 |
| Functors | 2 (F, G) | Sprint 2 |
| Tests | 400+ | Sprint 6 |
| Lines of Code | 12,000+ | Sprint 6 |
| Documentation | 15,000+ words | Sprint 6 |
| Binary Size (release) | <15 MB | Sprint 5 |

### Quality Targets

| Metric | Target |
|--------|--------|
| Test Coverage | >85% |
| Compilation Warnings | 0 |
| Clippy Warnings | 0 |
| Functor Law Compliance | 100% |
| Round-Trip Fidelity | 100% |

---

## Retrospective Template

After each sprint, document:

### What Went Well
- Successes
- Smooth implementations
- Learning moments

### What Could Be Improved
- Challenges
- Unexpected issues
- Technical debt

### Action Items
- Changes for next sprint
- Design improvements
- Process improvements

### Metrics
- Lines of code written
- Tests added
- Time spent
- Bugs found/fixed

---

## Next Steps

1. **Review this sprint plan** with stakeholders
2. **Begin Sprint 1** implementation
3. **Track progress** in PROGRESS.md
4. **Commit after each sprint** with detailed commit message
5. **Document retrospectives** after each sprint

---

**Status**: Ready to begin Sprint 1
**First Sprint**: Foundation & Value Objects
**Expected Timeline**: 6 sprints to completion
