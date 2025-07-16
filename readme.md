# CIM Domain Git

Git domain module for the Composable Information Machine (CIM) architecture.

## Overview

This module provides comprehensive Git repository introspection and graph extraction capabilities. It enables:

- Repository cloning and management
- Commit history analysis and visualization
- Branch and merge relationship mapping
- File change tracking and dependency analysis
- Integration with GitHub through MCP (Model Context Protocol)
- Configuration and deployment information extraction

## Architecture

The module follows Domain-Driven Design principles:

### Aggregates
- **Repository**: The aggregate root for Git repository operations
- **Commit**: Represents individual commits with metadata
- **Branch**: Manages branch lifecycle and relationships

### Value Objects
- **CommitHash**: Immutable Git commit identifiers
- **BranchName**: Validated branch names
- **RemoteUrl**: Git remote URLs
- **AuthorInfo**: Commit author information
- **FilePath**: Repository file paths

### Events
- **RepositoryCloned**: Repository successfully cloned
- **CommitAnalyzed**: Commit metadata extracted
- **BranchCreated/Deleted**: Branch lifecycle events
- **CommitGraphExtracted**: Commit graph generated
- **DependencyGraphExtracted**: File dependencies mapped

### Commands
- **CloneRepository**: Clone from remote URL
- **AnalyzeCommit**: Extract commit information
- **ExtractCommitGraph**: Build commit relationship graph
- **ExtractDependencyGraph**: Analyze file dependencies
- **GitHubIntegration**: Sync with GitHub via MCP

## Integration Points

### With cim-domain-graph
Converts Git structures (commits, branches, dependencies) into graph representations for visualization and analysis.

### With cim-domain-document
Extracts and processes documentation, configuration files, and deployment information from repositories.

### With cim-domain-agent
Enables agent-based Git operations through MCP, allowing AI agents to interact with GitHub repositories.

## Usage Examples

```rust
use cim_domain_git::{
    commands::CloneRepository,
    value_objects::RemoteUrl,
};

// Clone a repository
let cmd = CloneRepository {
    repository_id: None,
    remote_url: RemoteUrl::new("https://github.com/user/repo.git")?,
    local_path: "/tmp/repo".to_string(),
    branch: None,
    depth: None,
};

// Extract commit graph
let extract_cmd = ExtractCommitGraph {
    repository_id: repo_id,
    start_commit: None,
    max_depth: Some(100),
    include_all_branches: true,
    include_tags: true,
};
```

## Features

- **Event Sourcing**: All repository operations generate domain events
- **Graph Extraction**: Convert Git data to graph structures
- **GitHub Integration**: Connect to GitHub via MCP
- **Dependency Analysis**: Extract and analyze code dependencies
- **Configuration Mining**: Extract deployment and configuration data

## Development Status

This module is under active development. Current implementation includes:

- ✅ Core domain model (aggregates, value objects, events, commands)
- ⏳ Command handlers
- ⏳ Event projections
- ⏳ Query handlers
- ⏳ Git operations implementation
- ⏳ GitHub MCP integration

## License

MIT OR Apache-2.0 