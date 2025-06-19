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
//! - **cim-domain-graph**: Converts Git structures to graph representations
//! - **cim-domain-document**: Extracts and processes documentation from repositories
//! - **cim-domain-agent**: Enables agent-based Git operations through MCP

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
pub mod value_objects;

// Re-export commonly used types
pub use aggregate::{Repository, RepositoryId};
pub use commands::{AnalyzeCommit, CloneRepository};
pub use events::{CommitAnalyzed, RepositoryCloned};
pub use value_objects::{AuthorInfo, BranchName, CommitHash};

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

    /// Infrastructure error
    #[error("Infrastructure error: {0}")]
    InfrastructureError(#[from] anyhow::Error),
}

/// Result type for Git domain operations
pub type Result<T> = std::result::Result<T, GitDomainError>;
