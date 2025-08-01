// Copyright 2025 Cowboy AI, LLC.

//! # CIM Git Domain
//!
//! This module provides Git repository introspection and graph extraction capabilities
//! for the Composable Information Machine (CIM) architecture.
//!
//! ## Overview
//!
//! The Git domain module enables:
//! - Repository analysis and introspection
//! - Commit graph extraction and visualization
//! - Branch and merge relationship mapping
//! - File change tracking and dependency analysis
//! - Integration with GitHub through MCP (Model Context Protocol)
//! - Configuration and deployment information extraction
//!
//! ## Architecture
//!
//! The module follows Domain-Driven Design principles with:
//! - **Aggregates**: Repository, Commit, Branch
//! - **Value Objects**: `CommitHash`, `BranchName`, `AuthorInfo`
//! - **Events**: `RepositoryCloned`, `CommitAnalyzed`, `BranchCreated`
//! - **Commands**: `CloneRepository`, `AnalyzeCommit`, `ExtractGraph`
//!
//! ## Integration Points
//!
//! - **Graph Domain**: Converts Git structures to graph representations (temporarily using local types)
//! - **Document Domain**: Extracts and processes documentation from repositories (pending integration)
//! - **Agent Domain**: Enables agent-based Git operations through MCP (pending integration)
//!
//! ## Examples
//!
//! ### Creating a Repository
//!
//! ```
//! use cim_domain_git::aggregate::Repository;
//! use cim_domain_git::value_objects::{RemoteUrl, BranchName};
//!
//! // Create a new repository aggregate
//! let mut repo = Repository::new("my-project".to_string());
//!
//! // Clone from a remote URL
//! let remote_url = RemoteUrl::new("https://github.com/user/repo.git").unwrap();
//! let events = repo.clone_repository(remote_url, "/tmp/repo".to_string()).unwrap();
//!
//! assert_eq!(events.len(), 1);
//! assert!(repo.local_path.is_some());
//! ```
//!
//! ### Working with Commit Hashes
//!
//! ```
//! use cim_domain_git::value_objects::CommitHash;
//!
//! // Create a commit hash
//! let hash = CommitHash::new("abc123def456789").unwrap();
//! assert_eq!(hash.short(), "abc123d");
//!
//! // Invalid hashes are rejected
//! assert!(CommitHash::new("not-hex").is_err());
//! assert!(CommitHash::new("short").is_err());
//! ```
//!
//! ### Creating Commands
//!
//! ```
//! use cim_domain_git::commands::{CloneRepository, AnalyzeCommit};
//! use cim_domain_git::aggregate::RepositoryId;
//! use cim_domain_git::value_objects::{RemoteUrl, CommitHash};
//!
//! // Clone repository command
//! let clone_cmd = CloneRepository {
//!     repository_id: None,
//!     remote_url: RemoteUrl::new("https://github.com/rust-lang/rust.git").unwrap(),
//!     local_path: "/workspace/rust".to_string(),
//!     branch: None,
//!     depth: Some(1), // Shallow clone
//! };
//!
//! // Analyze commit command
//! let repo_id = RepositoryId::new();
//! let analyze_cmd = AnalyzeCommit {
//!     repository_id: repo_id,
//!     commit_hash: CommitHash::new("1234567890abcdef").unwrap(),
//!     analyze_files: true,
//!     extract_dependencies: true,
//! };
//! ```
//!
//! ### Error Handling
//!
//! ```
//! use cim_domain_git::{GitDomainError, Result};
//! use cim_domain_git::value_objects::CommitHash;
//!
//! fn validate_commit(hash_str: &str) -> Result<CommitHash> {
//!     match CommitHash::new(hash_str) {
//!         Ok(hash) => Ok(hash),
//!         Err(e) => {
//!             eprintln!("Invalid commit hash: {e}");
//!             Err(e)
//!         }
//!     }
//! }
//!
//! // Valid hash
//! assert!(validate_commit("abc123def456789").is_ok());
//!
//! // Invalid hash
//! assert!(matches!(
//!     validate_commit("invalid"),
//!     Err(GitDomainError::InvalidCommitHash(_))
//! ));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod security;
pub mod value_objects;
// TODO: dependency_analysis module has been disabled
// This was used by the removed graph extraction functionality
// pub mod dependency_analysis;
pub mod cache;

// Re-export commonly used types
pub use aggregate::{Repository, RepositoryId};
pub use commands::{AnalyzeCommit, CloneRepository};
pub use events::{CommitAnalyzed, RepositoryCloned};
pub use value_objects::{AuthorInfo, BranchName, CommitHash};

// Re-export projections
pub use projections::{
    RepositoryListProjection, RepositorySummary,
    CommitHistoryProjection, CommitHistoryEntry,
    BranchStatusProjection, BranchInfo,
    FileChangeProjection, FileChange, FileStatistics,
    ProjectionError,
};

// Re-export queries
pub use queries::{
    GitQueryHandler, QueryError,
    GetRepositoryDetails, RepositoryDetailsResult,
    GetCommitHistory, CommitHistoryResult,
    GetBranchList, BranchListResult,
    ListRepositories, ListRepositoriesResult,
};

/// Domain-specific errors for Git operations
#[derive(Debug, thiserror::Error)]
pub enum GitDomainError {
    /// Repository not found
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    /// Invalid commit hash
    #[error("Invalid commit hash: {0}")]
    InvalidCommitHash(String),

    /// Git operation failed
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),

    /// Graph extraction failed
    #[error("Graph extraction failed: {0}")]
    GraphExtractionFailed(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Infrastructure error
    #[error("Infrastructure error: {0}")]
    InfrastructureError(#[from] anyhow::Error),
}

/// Result type for Git domain operations
pub type Result<T> = std::result::Result<T, GitDomainError>;
