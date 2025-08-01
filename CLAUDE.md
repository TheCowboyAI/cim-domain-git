<!-- Copyright 2025 Cowboy AI, LLC. -->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Build**: `cargo build`
- **Test**: `cargo test` or `cargo nextest run`
- **Run Single Test**: `cargo test test_name` or `cargo nextest run test_name`
- **Lint**: `cargo clippy`
- **Format Check**: `cargo fmt --check`
- **Format Fix**: `cargo fmt`

### Nix Development (in nix develop shell)
- **Enter Dev Shell**: `nix develop`
- **Build with Nix**: `nix build`
- **Update flake.lock**: `nix flake update`
- **Check flake**: `nix flake check`

### Examples
- **Git to Graph Demo**: `cargo run --example git_to_graph` - Demonstrates converting Git repository data to graph structures

## Architecture

This is the Git Domain module within the CIM (Composable Information Machine) ecosystem. It provides comprehensive Git repository introspection, analysis, and graph extraction capabilities.

### Core Design Principles
1. **Event-Driven Architecture**: All state changes are events following CIM patterns
2. **CQRS Pattern**: Separate command handlers (`handlers/commands/`) and query handlers (`handlers/queries/`)
3. **Domain Isolation**: No shared state with other domains
4. **Security First**: Path traversal protection, command injection prevention
5. **Performance Optimized**: Caching layer for expensive Git operations

### Key Components

**Aggregates** (in `aggregate/`):
- `RepositoryAggregate`: Manages Git repository lifecycle and state
- `CommitAggregate`: Handles individual commit state and relationships
- `BranchAggregate`: Manages branch operations and tracking

**Value Objects** (in `value_objects/`):
- `CommitHash`: Immutable SHA-1 commit identifiers with validation
- `BranchName`: Validated branch names (e.g., `main`, `feature/xyz`)
- `RemoteUrl`: Git remote URLs with security validation
- `FilePath`: Repository-relative file paths
- `AuthorInfo`: Commit author information (name, email)

**Commands & Queries**:
- Commands: `CloneRepository`, `AnalyzeCommit`
- Queries: `GetRepositoryDetails`, `GetCommitHistory`, `ListRepositories`, `GetFileChanges`

**Services** (in root):
- `dependency_analysis.rs`: Multi-language dependency extraction (Rust, Python, JS/TS, Java, Go, C/C++)
- `cache.rs`: Performance optimization for Git operations
- `security.rs`: Path traversal and command injection protection

### Event Flow
1. Commands are dispatched through CQRS handlers
2. Handlers interact with Git repositories via `git2` library
3. Domain events are emitted (e.g., `RepositoryCloned`, `CommitAnalyzed`, `BranchCreated`)
4. Events include correlation and causation IDs per CIM standards
5. Projections maintain read models from event streams

### Integration Points
- **cim-domain**: Core CIM types (DomainEvent, Aggregate, etc.)
- **cim-subject**: Event routing and NATS subjects
- **cim-ipld**: IPLD data structures for content-addressable storage
- **git2**: Low-level Git operations

Note: This domain is designed to be independent and composable. Integration with other domains (graph, document, agent) should be handled at the composition layer, not within this domain.

## Important Patterns

### Event Sourcing
All events MUST follow CIM event sourcing patterns:
```rust
// Events must have correlation and causation IDs
pub struct RepositoryCloned {
    pub repository_id: RepositoryId,
    pub remote_url: RemoteUrl,
    pub local_path: String,
    pub cloned_at: DateTime<Utc>,
    // From DomainEvent trait
    pub correlation_id: String,
    pub causation_id: String,
}
```

### NATS Subjects
Follow CIM subject naming conventions:
- Commands: `git.repository.clone`, `git.commit.analyze`
- Events: `git.repository.cloned`, `git.commit.analyzed`, `git.branch.created`
- Queries: `git.repository.get`, `git.commit.list`, `git.branch.list`

### Security Patterns
Always validate paths and URLs:
```rust
// Use SecurityValidator for all external inputs
let validated_path = SecurityValidator::validate_path(&path)?;
let safe_url = SecurityValidator::validate_git_url(&url)?;
```

### Dependency Analysis
The system supports multiple languages through pattern matching:
- **Rust**: Parses `use` and `extern crate` statements
- **Python**: Parses `import` and `from ... import` statements
- **JavaScript/TypeScript**: Parses `import`, `require()`, and dynamic imports
- **Java**: Parses `import` statements
- **Go**: Parses `import` statements
- **C/C++**: Parses `#include` directives

## Testing Requirements
- **Integration Tests**: Test with real Git repositories using `tempfile`
- **Property Testing**: Use `proptest` for value object validation
- **Mock External Calls**: Mock GitHub API calls in tests
- **Cache Testing**: Verify cache behavior with time-based expiration
- **Security Testing**: Test path traversal and injection attack prevention