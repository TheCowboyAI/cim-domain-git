<!-- Copyright 2025 Cowboy AI, LLC. -->

# CIM Domain Git

[![CI](https://github.com/thecowboyai/cim-domain-git/actions/workflows/ci.yml/badge.svg)](https://github.com/thecowboyai/cim-domain-git/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cim-domain-git.svg)](https://crates.io/crates/cim-domain-git)
[![Documentation](https://docs.rs/cim-domain-git/badge.svg)](https://docs.rs/cim-domain-git)
[![Test Coverage](https://img.shields.io/codecov/c/github/thecowboyai/cim-domain-git)](https://codecov.io/gh/thecowboyai/cim-domain-git)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Git domain module for the Composable Information Machine (CIM) - provides comprehensive Git repository introspection and analysis capabilities.

## Overview

The `cim-domain-git` crate provides Git repository operations within the CIM ecosystem, following Domain-Driven Design patterns with event sourcing and CQRS:

- **Repository Management**: Clone, analyze, and track Git repositories
- **Commit Analysis**: Extract and analyze commit metadata and relationships
- **Branch Operations**: Track branch lifecycle and relationships
- **Dependency Analysis**: Multi-language dependency extraction
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

#### Domain Events
- **RepositoryCloned**: Repository successfully cloned
- **CommitAnalyzed**: Commit metadata extracted
- **BranchCreated/Deleted**: Branch lifecycle events
- **RepositoryAnalyzed**: Full repository analysis complete

#### Commands & Queries
- **Commands**: `CloneRepository`, `AnalyzeCommit`
- **Queries**: `GetRepositoryDetails`, `GetCommitHistory`, `ListRepositories`

## Features

- **Event Sourcing**: All operations generate domain events with correlation IDs
- **CQRS Pattern**: Separate command and query handlers
- **Multi-Language Support**: Dependency analysis for Rust, Python, JS/TS, Java, Go, C/C++
- **Security First**: Built-in protection against path traversal and command injection
- **Performance**: Caching layer with configurable TTL
- **Async Operations**: Full async/await support with Tokio

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

### Analyzing Commits

```rust
use cim_domain_git::{
    commands::AnalyzeCommit,
    value_objects::CommitHash,
};

let analyze_cmd = AnalyzeCommit {
    repository_id: repo_id,
    commit_hash: CommitHash::new("abc123def456")?,
    analyze_files: true,
    extract_dependencies: true,
};
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

## Integration

This domain is designed to be independent and composable. Integration with other CIM domains (graph, document, agent) should be handled at the composition layer, not within this domain.

### Dependencies
- `cim-domain`: Core CIM types and traits
- `cim-subject`: Event routing with NATS subjects
- `cim-ipld`: Content-addressable storage
- `git2`: Low-level Git operations

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
cargo nextest run  # If using nextest
```

### Examples

```bash
cargo run --example git_to_graph  # When composition example is available
```

## Status

This module is production-ready with comprehensive test coverage:
- ✅ Core domain model (aggregates, value objects, events, commands)
- ✅ Value object validation and security
- ✅ Event sourcing infrastructure
- ✅ Caching layer
- ✅ Multi-language dependency analysis
- 🚧 Command handlers (in progress)
- 🚧 Query handlers (in progress)
- 🚧 GitHub MCP integration (planned)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built as part of the Composable Information Machine (CIM) ecosystem by Cowboy AI, LLC.