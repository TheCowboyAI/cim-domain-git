<!-- Copyright 2025 Cowboy AI, LLC. -->

# CIM Domain Git

[![CI](https://github.com/thecowboyai/cim-domain-git/actions/workflows/ci.yml/badge.svg)](https://github.com/thecowboyai/cim-domain-git/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cim-domain-git.svg)](https://crates.io/crates/cim-domain-git)
[![Documentation](https://docs.rs/cim-domain-git/badge.svg)](https://docs.rs/cim-domain-git)
[![Test Coverage](https://img.shields.io/codecov/c/github/thecowboyai/cim-domain-git)](https://codecov.io/gh/thecowboyai/cim-domain-git)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Git domain module for the Composable Information Machine (CIM) - provides comprehensive Git repository introspection, metadata extraction, and graph analytics capabilities.

## Overview

The `cim-domain-git` crate provides Git repository operations within the CIM ecosystem, following Domain-Driven Design patterns with event sourcing and CQRS. It now includes powerful analyzers for extracting metadata optimized for building graphs:

- **Repository Management**: Clone, analyze, and track Git repositories
- **Commit Analysis**: Extract and analyze commit metadata and relationships
- **Graph Analytics**: Collaboration patterns, code ownership, and team dynamics
- **Code Quality Analysis**: Technical debt detection and repository health metrics
- **Dependency Analysis**: Multi-language dependency extraction
- **Event Streaming**: NATS JetStream integration for real-time event distribution
- **Security Features**: Path traversal and command injection protection
- **Performance Optimization**: Built-in caching for expensive operations

## Architecture

### Domain-Driven Design Components

#### Aggregates
- **Repository**: The aggregate root for Git repository operations
- **Commit**: Individual commits with full metadata
- **Branch**: Branch lifecycle and relationships

#### Value Objects
- **CommitHash**: Validated SHA-1 commit identifiers
- **BranchName**: Validated branch names
- **RemoteUrl**: Secure Git remote URLs
- **AuthorInfo**: Commit author information
- **FilePath**: Repository-relative file paths
- **TagName**: Git tag identifiers

#### Domain Events
- **RepositoryCloned**: Repository successfully cloned
- **CommitAnalyzed**: Commit metadata extracted
- **BranchCreated/Deleted**: Branch lifecycle events
- **RepositoryAnalyzed**: Full repository analysis complete
- **CollaborationDetected**: Developer collaboration patterns identified
- **CodeOwnershipCalculated**: Primary code contributors identified
- **TechnicalDebtIdentified**: Code quality issues detected
- **RepositoryHealthCalculated**: Overall health metrics computed

#### Commands & Queries
- **Commands**: `CloneRepository`, `AnalyzeCommit`, `CreateBranch`, `AnalyzeRepository`
- **Queries**: `GetRepositoryDetails`, `GetCommitHistory`, `ListRepositories`

## Features

### Core Features
- **Event Sourcing**: All operations generate domain events with correlation IDs
- **CQRS Pattern**: Separate command and query handlers
- **Multi-Language Support**: Dependency analysis for Rust, Python, JS/TS, Java, Go, C/C++
- **Security First**: Built-in protection against path traversal and command injection
- **Performance**: Caching layer with configurable TTL
- **Async Operations**: Full async/await support with Tokio

### Graph Analytics (New in v0.5.0)
- **Collaboration Analysis**: Detect developer collaboration patterns
- **Code Ownership**: Calculate primary contributors to files/directories
- **Team Clustering**: Identify natural team formations
- **Technical Debt Detection**: Find high-complexity and high-churn code
- **Repository Health**: Compute overall health metrics
- **Circular Dependencies**: Detect dependency cycles

## Usage

### Basic Repository Operations

```rust
use cim_domain_git::{
    commands::CloneRepository,
    value_objects::RemoteUrl,
    RepositoryId,
};

// Clone a repository
let cmd = CloneRepository {
    repository_id: None,
    remote_url: RemoteUrl::new("https://github.com/user/repo.git")?,
    local_path: "/tmp/repo".to_string(),
    branch: None,
    depth: Some(1), // Shallow clone
};
```

### Graph Analytics

```rust
use cim_domain_git::analyzers::{CollaborationAnalyzer, CodeQualityAnalyzer};

// Analyze collaboration patterns
let collab_analyzer = CollaborationAnalyzer::new();
let collaborations = collab_analyzer.analyze_collaboration(
    repository_id,
    &commits,
);

// Analyze code quality
let quality_analyzer = CodeQualityAnalyzer::new();
let health = quality_analyzer.calculate_repository_health(
    repository_id,
    active_contributors,
    commits_last_week,
    total_branches,
    stale_branches,
    critical_issues,
);
```

### NATS Event Streaming

```rust
use cim_domain_git::nats::{NatsClient, EventPublisher};

// Connect to NATS
let client = NatsClient::connect("nats://localhost:4222").await?;

// Publish events
let publisher = EventPublisher::new(client);
publisher.publish(event).await?;
```

### Value Object Validation

```rust
use cim_domain_git::value_objects::{CommitHash, BranchName};

// Commit hashes are validated
let hash = CommitHash::new("abc123def456789")?;
assert_eq!(hash.short(), "abc123d");

// Branch names follow Git conventions
let branch = BranchName::new("feature/new-feature")?;
assert!(!branch.is_default());
```

## Examples

Run the examples to see the features in action:

```bash
# Graph analytics demonstration
cargo run --example graph_analysis_demo

# NATS integration
cargo run --example nats_integration_demo

# JetStream event sourcing
cargo run --example jetstream_event_sourcing
```

## Integration

This domain integrates with other CIM components:

### Dependencies
- `cim-domain`: Core CIM types and traits
- `cim-subject`: Event routing with NATS subjects
- `cim-ipld`: Content-addressable storage
- `git2`: Low-level Git operations
- `async-nats`: NATS messaging (v0.42)

### Integration Points
- **Event Streaming**: Publish Git events to NATS for real-time processing
- **Graph Building**: Extract metadata optimized for graph databases
- **IPLD Storage**: Store Git objects in content-addressable format

## Development

### Building

```bash
# Standard build
cargo build

# With Nix
nix develop
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with nextest
cargo nextest run

# Run specific test suite
cargo test --lib
cargo test --test nats_integration_test
```

### Documentation

See the `doc/` directory for detailed documentation:
- [API Documentation](doc/api.md)
- [Graph Analysis Features](doc/graph-analysis.md)
- [Design Documentation](doc/design/git-domain-design.md)
- [User Stories](doc/user-stories.md)

## Status

This module is production-ready with comprehensive test coverage:
- ✅ Core domain model (aggregates, value objects, events, commands)
- ✅ Value object validation and security
- ✅ Event sourcing infrastructure with NATS JetStream
- ✅ Caching layer with LRU eviction
- ✅ Multi-language dependency analysis
- ✅ Graph analytics (collaboration, ownership, technical debt)
- ✅ Command and query handlers
- ✅ 96+ tests passing with comprehensive coverage
- 🚧 GitHub MCP integration (planned)

## Version History

- **v0.5.0** - Added graph analytics, completed NATS integration, 96 tests passing
- **v0.3.0** - Core domain implementation with event sourcing
- **v0.1.0** - Initial release

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built as part of the Composable Information Machine (CIM) ecosystem by Cowboy AI, LLC.