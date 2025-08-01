// Copyright 2025 Cowboy AI, LLC.

//! Aggregates for the Git domain
//!
//! This module contains the aggregate roots that maintain consistency
//! boundaries for Git-related operations.

use crate::GitDomainError;
use crate::events::{GitDomainEvent, RepositoryCloned};
use crate::value_objects::{AuthorInfo, BranchName, CommitHash, RemoteUrl};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a repository
///
/// # Examples
///
/// ```
/// use cim_domain_git::aggregate::RepositoryId;
/// use uuid::Uuid;
///
/// // Create a new repository ID
/// let id = RepositoryId::new();
///
/// // Create from existing UUID
/// let uuid = Uuid::new_v4();
/// let id_from_uuid = RepositoryId::from_uuid(uuid);
/// assert_eq!(id_from_uuid.as_uuid(), &uuid);
///
/// // IDs can be compared
/// let id1 = RepositoryId::new();
/// let id2 = RepositoryId::new();
/// assert_ne!(id1, id2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId(Uuid);

impl RepositoryId {
    /// Create a new repository ID
    #[must_use] pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    #[must_use] pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    #[must_use] pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for RepositoryId {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository aggregate root
///
/// The Repository aggregate maintains the consistency boundary for all
/// repository-related operations including cloning, branch management,
/// and commit analysis.
///
/// # Examples
///
/// ```
/// use cim_domain_git::aggregate::Repository;
/// use cim_domain_git::value_objects::RemoteUrl;
///
/// // Create a new repository
/// let mut repo = Repository::new("awesome-project".to_string());
/// assert_eq!(repo.metadata.name, "awesome-project");
/// assert_eq!(repo.version, 0);
///
/// // Clone the repository
/// let remote_url = RemoteUrl::new("https://github.com/example/repo.git").unwrap();
/// let events = repo.clone_repository(remote_url, "/workspace/repo".to_string()).unwrap();
///
/// // Repository state is updated
/// assert!(repo.remote_url.is_some());
/// assert_eq!(repo.local_path, Some("/workspace/repo".to_string()));
/// assert_eq!(repo.version, 1);
/// ```
///
/// ## Event Sourcing
///
/// ```
/// use cim_domain_git::aggregate::Repository;
/// use cim_domain_git::events::{GitDomainEvent, RepositoryCloned};
/// use cim_domain_git::value_objects::RemoteUrl;
/// use chrono::Utc;
///
/// let mut repo = Repository::new("test-repo".to_string());
///
/// // Apply an event
/// let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
///     repository_id: repo.id,
///     remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
///     local_path: "/tmp/test".to_string(),
///     timestamp: Utc::now(),
/// });
///
/// repo.apply_event(&event).unwrap();
/// assert_eq!(repo.version, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique identifier
    pub id: RepositoryId,

    /// Remote URL of the repository
    pub remote_url: Option<RemoteUrl>,

    /// Local path where repository is cloned
    pub local_path: Option<String>,

    /// Current HEAD commit
    pub head: Option<CommitHash>,

    /// Branches in the repository
    pub branches: HashMap<BranchName, CommitHash>,

    /// Repository metadata
    pub metadata: RepositoryMetadata,

    /// Aggregate version for optimistic locking
    pub version: u64,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Repository name
    pub name: String,

    /// Repository description
    pub description: Option<String>,

    /// Primary language
    pub primary_language: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Repository size in bytes
    pub size_bytes: Option<u64>,

    /// Number of commits
    pub commit_count: Option<usize>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl Repository {
    /// Create a new repository aggregate
    #[must_use] pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: RepositoryId::new(),
            remote_url: None,
            local_path: None,
            head: None,
            branches: HashMap::new(),
            metadata: RepositoryMetadata {
                name,
                description: None,
                primary_language: None,
                created_at: now,
                updated_at: now,
                size_bytes: None,
                commit_count: None,
                custom: HashMap::new(),
            },
            version: 0,
        }
    }

    /// Handle a clone repository command
    pub fn clone_repository(
        &mut self,
        remote_url: RemoteUrl,
        local_path: String,
    ) -> Result<Vec<GitDomainEvent>, GitDomainError> {
        if self.local_path.is_some() {
            return Err(GitDomainError::GitOperationFailed(
                "Repository already cloned".to_string(),
            ));
        }

        let event = RepositoryCloned {
            repository_id: self.id,
            remote_url: remote_url.clone(),
            local_path: local_path.clone(),
            timestamp: Utc::now(),
        };

        self.apply_event(&GitDomainEvent::RepositoryCloned(event.clone()))?;

        Ok(vec![GitDomainEvent::RepositoryCloned(event)])
    }

    /// Apply an event to update the aggregate state
    pub fn apply_event(&mut self, event: &GitDomainEvent) -> Result<(), GitDomainError> {
        match event {
            GitDomainEvent::RepositoryCloned(e) => {
                self.remote_url = Some(e.remote_url.clone());
                self.local_path = Some(e.local_path.clone());
                self.metadata.updated_at = e.timestamp;
            }
            GitDomainEvent::CommitAnalyzed(e) => {
                self.metadata.commit_count = Some(self.metadata.commit_count.unwrap_or(0) + 1);
                self.metadata.updated_at = e.timestamp;
            }
            GitDomainEvent::BranchCreated(e) => {
                self.branches
                    .insert(e.branch_name.clone(), e.commit_hash.clone());
                self.metadata.updated_at = e.timestamp;
            }
            _ => {} // Handle other events as needed
        }

        self.version += 1;
        Ok(())
    }
}

/// Commit aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Repository this commit belongs to
    pub repository_id: RepositoryId,

    /// Commit hash
    pub hash: CommitHash,

    /// Parent commits
    pub parents: Vec<CommitHash>,

    /// Commit author
    pub author: AuthorInfo,

    /// Commit timestamp
    pub timestamp: DateTime<Utc>,

    /// Commit message
    pub message: String,

    /// Files changed in this commit
    pub files_changed: Vec<FileChange>,
}

/// Represents a file change in a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: String,

    /// Change type
    pub change_type: ChangeType,

    /// Lines added
    pub additions: usize,

    /// Lines deleted
    pub deletions: usize,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

/// Branch aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Repository this branch belongs to
    pub repository_id: RepositoryId,

    /// Branch name
    pub name: BranchName,

    /// Current commit this branch points to
    pub head: CommitHash,

    /// Whether this is the default branch
    pub is_default: bool,

    /// Branch metadata
    pub metadata: BranchMetadata,
}

/// Branch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Number of commits ahead of default branch
    pub ahead_count: Option<usize>,

    /// Number of commits behind default branch
    pub behind_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        let repo = Repository::new("test-repo".to_string());
        assert_eq!(repo.metadata.name, "test-repo");
        assert!(repo.remote_url.is_none());
        assert!(repo.local_path.is_none());
        assert_eq!(repo.version, 0);
    }

    #[test]
    fn test_repository_clone() {
        let mut repo = Repository::new("test-repo".to_string());
        let remote_url = RemoteUrl::new("https://github.com/test/repo.git").unwrap();
        let local_path = "/tmp/test-repo".to_string();

        let events = repo
            .clone_repository(remote_url.clone(), local_path.clone())
            .unwrap();

        assert_eq!(events.len(), 1);
        assert!(repo.remote_url.is_some());
        assert_eq!(repo.local_path, Some(local_path));
        assert_eq!(repo.version, 1);
    }
}
