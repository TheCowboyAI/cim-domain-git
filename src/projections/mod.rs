// Copyright 2025 Cowboy AI, LLC.

//! Read model projections for the Git domain
//!
//! This module contains projections that build read models
//! from the event stream for efficient querying.

use crate::aggregate::RepositoryId;
use crate::events::GitDomainEvent;
use crate::value_objects::{BranchName, CommitHash, RemoteUrl, AuthorInfo, FilePath};
use crate::events::{FileChangeType};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Repository summary for list views
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositorySummary {
    /// Repository ID
    pub id: RepositoryId,
    /// Repository name
    pub name: String,
    /// Remote URL if cloned
    pub remote_url: Option<RemoteUrl>,
    /// Local path if cloned
    pub local_path: Option<String>,
    /// Number of branches
    pub branch_count: usize,
    /// Number of commits analyzed
    pub commit_count: usize,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Projection that maintains a list of all repositories
pub struct RepositoryListProjection {
    repositories: Arc<RwLock<HashMap<RepositoryId, RepositorySummary>>>,
}

impl RepositoryListProjection {
    /// Create a new repository list projection
    #[must_use] pub fn new() -> Self {
        Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle a domain event to update the projection
    pub fn handle_event(&self, event: &GitDomainEvent) -> Result<(), ProjectionError> {
        let mut repos = self.repositories.write()
            .map_err(|_| ProjectionError::LockPoisoned)?;

        match event {
            GitDomainEvent::RepositoryCloned(e) => {
                let summary = repos.entry(e.repository_id)
                    .or_insert_with(|| RepositorySummary {
                        id: e.repository_id,
                        name: e.local_path.split('/').next_back()
                            .unwrap_or("unknown")
                            .to_string(),
                        remote_url: None,
                        local_path: None,
                        branch_count: 0,
                        commit_count: 0,
                        last_updated: e.timestamp,
                    });
                
                summary.remote_url = Some(e.remote_url.clone());
                summary.local_path = Some(e.local_path.clone());
                summary.last_updated = e.timestamp;
            }
            GitDomainEvent::RepositoryAnalyzed(e) => {
                let summary = repos.entry(e.repository_id)
                    .or_insert_with(|| RepositorySummary {
                        id: e.repository_id,
                        name: e.name.clone(),
                        remote_url: None,
                        local_path: Some(e.path.clone()),
                        branch_count: 0,
                        commit_count: 0,
                        last_updated: e.timestamp,
                    });
                
                summary.name = e.name.clone();
                summary.local_path = Some(e.path.clone());
                summary.branch_count = e.branch_count;
                summary.commit_count = e.commit_count;
                summary.last_updated = e.timestamp;
            }
            GitDomainEvent::BranchCreated(e) => {
                if let Some(summary) = repos.get_mut(&e.repository_id) {
                    summary.branch_count += 1;
                    summary.last_updated = e.timestamp;
                }
            }
            GitDomainEvent::CommitAnalyzed(e) => {
                if let Some(summary) = repos.get_mut(&e.repository_id) {
                    summary.commit_count += 1;
                    summary.last_updated = e.timestamp;
                }
            }
            _ => {} // Other events don't affect the list view
        }

        Ok(())
    }

    /// Get all repositories
    pub fn get_all(&self) -> Result<Vec<RepositorySummary>, ProjectionError> {
        let repos = self.repositories.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        Ok(repos.values().cloned().collect())
    }

    /// Get a specific repository summary
    pub fn get_by_id(&self, id: &RepositoryId) -> Result<Option<RepositorySummary>, ProjectionError> {
        let repos = self.repositories.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        Ok(repos.get(id).cloned())
    }

    /// Get repositories by remote URL pattern
    pub fn find_by_remote_url(&self, pattern: &str) -> Result<Vec<RepositorySummary>, ProjectionError> {
        let repos = self.repositories.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        Ok(repos.values()
            .filter(|r| r.remote_url.as_ref()
                .is_some_and(|url| url.as_str().contains(pattern)))
            .cloned()
            .collect())
    }
}

impl Default for RepositoryListProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Commit history entry for detailed views
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitHistoryEntry {
    /// Commit hash
    pub hash: CommitHash,
    /// Parent commits
    pub parents: Vec<CommitHash>,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit message
    pub message: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of files changed
    pub files_changed: usize,
}

/// Projection that maintains commit history for repositories
pub struct CommitHistoryProjection {
    /// Map of repository ID to commit history
    commits: Arc<RwLock<HashMap<RepositoryId, Vec<CommitHistoryEntry>>>>,
}

impl CommitHistoryProjection {
    /// Create a new commit history projection
    #[must_use] pub fn new() -> Self {
        Self {
            commits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle a domain event to update the projection
    pub fn handle_event(&self, event: &GitDomainEvent) -> Result<(), ProjectionError> {
        if let GitDomainEvent::CommitAnalyzed(e) = event {
            let mut commits = self.commits.write()
                .map_err(|_| ProjectionError::LockPoisoned)?;
            
            let history = commits.entry(e.repository_id)
                .or_insert_with(Vec::new);
            
            history.push(CommitHistoryEntry {
                hash: e.commit_hash.clone(),
                parents: e.parents.clone(),
                author_name: e.author.name.clone(),
                author_email: e.author.email.clone(),
                message: e.message.clone(),
                timestamp: e.commit_timestamp,
                files_changed: e.files_changed.len(),
            });
            
            // Keep commits sorted by timestamp (newest first)
            history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
        
        Ok(())
    }

    /// Get commit history for a repository
    pub fn get_history(&self, repository_id: &RepositoryId, limit: Option<usize>) -> Result<Vec<CommitHistoryEntry>, ProjectionError> {
        let commits = self.commits.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        if let Some(history) = commits.get(repository_id) {
            let result = if let Some(limit) = limit {
                history.iter().take(limit).cloned().collect()
            } else {
                history.clone()
            };
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get a specific commit
    pub fn get_commit(&self, repository_id: &RepositoryId, hash: &CommitHash) -> Result<Option<CommitHistoryEntry>, ProjectionError> {
        let commits = self.commits.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        if let Some(history) = commits.get(repository_id) {
            Ok(history.iter().find(|c| &c.hash == hash).cloned())
        } else {
            Ok(None)
        }
    }
}

impl Default for CommitHistoryProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Branch information for branch status views
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: BranchName,
    /// Current commit
    pub head: CommitHash,
    /// Whether this is the default branch
    pub is_default: bool,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Projection that maintains branch status for repositories
pub struct BranchStatusProjection {
    /// Map of repository ID to branches
    branches: Arc<RwLock<HashMap<RepositoryId, HashMap<BranchName, BranchInfo>>>>,
}

impl BranchStatusProjection {
    /// Create a new branch status projection
    #[must_use] pub fn new() -> Self {
        Self {
            branches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle a domain event to update the projection
    pub fn handle_event(&self, event: &GitDomainEvent) -> Result<(), ProjectionError> {
        if let GitDomainEvent::BranchCreated(e) = event {
            let mut branches = self.branches.write()
                .map_err(|_| ProjectionError::LockPoisoned)?;
            
            let repo_branches = branches.entry(e.repository_id)
                .or_insert_with(HashMap::new);
            
            repo_branches.insert(e.branch_name.clone(), BranchInfo {
                name: e.branch_name.clone(),
                head: e.commit_hash.clone(),
                is_default: e.branch_name.is_default(),
                last_updated: e.timestamp,
            });
        }
        
        Ok(())
    }

    /// Get all branches for a repository
    pub fn get_branches(&self, repository_id: &RepositoryId) -> Result<Vec<BranchInfo>, ProjectionError> {
        let branches = self.branches.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        if let Some(repo_branches) = branches.get(repository_id) {
            Ok(repo_branches.values().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get a specific branch
    pub fn get_branch(&self, repository_id: &RepositoryId, name: &BranchName) -> Result<Option<BranchInfo>, ProjectionError> {
        let branches = self.branches.read()
            .map_err(|_| ProjectionError::LockPoisoned)?;
        
        if let Some(repo_branches) = branches.get(repository_id) {
            Ok(repo_branches.get(name).cloned())
        } else {
            Ok(None)
        }
    }
}

impl Default for BranchStatusProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// File change tracking projection
///
/// This projection tracks file changes across commits
/// to provide efficient file history queries.
#[derive(Debug, Clone)]
pub struct FileChangeProjection {
    /// File changes indexed by file path
    file_changes: Arc<RwLock<HashMap<FilePath, Vec<FileChange>>>>,
    /// File changes indexed by commit
    commit_changes: Arc<RwLock<HashMap<CommitHash, Vec<FileChange>>>>,
    /// File rename tracking
    rename_history: Arc<RwLock<HashMap<FilePath, Vec<RenameInfo>>>>,
}

/// Individual file change record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChange {
    /// File path
    pub path: FilePath,
    /// Commit where change occurred
    pub commit_hash: CommitHash,
    /// Type of change
    pub change_type: FileChangeType,
    /// Lines added
    pub additions: usize,
    /// Lines deleted
    pub deletions: usize,
    /// Author of the change
    pub author: AuthorInfo,
    /// Timestamp of the change
    pub timestamp: DateTime<Utc>,
}

/// File rename information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenameInfo {
    /// Original path
    pub old_path: FilePath,
    /// New path
    pub new_path: FilePath,
    /// Commit where rename occurred
    pub commit_hash: CommitHash,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl FileChangeProjection {
    /// Create a new file change projection
    #[must_use] pub fn new() -> Self {
        Self {
            file_changes: Arc::new(RwLock::new(HashMap::new())),
            commit_changes: Arc::new(RwLock::new(HashMap::new())),
            rename_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle domain events
    pub async fn handle_event(&self, event: &GitDomainEvent) -> Result<(), ProjectionError> {
        match event {
            GitDomainEvent::CommitAnalyzed(event) => {
                let mut file_changes = self.file_changes.write()
                    .map_err(|_| ProjectionError::LockError)?;
                let mut commit_changes = self.commit_changes.write()
                    .map_err(|_| ProjectionError::LockError)?;
                let mut rename_history = self.rename_history.write()
                    .map_err(|_| ProjectionError::LockError)?;

                let mut changes_for_commit = Vec::new();

                for file_change_info in &event.files_changed {
                    let change = FileChange {
                        path: file_change_info.path.clone(),
                        commit_hash: event.commit_hash.clone(),
                        change_type: file_change_info.change_type,
                        additions: file_change_info.additions,
                        deletions: file_change_info.deletions,
                        author: event.author.clone(),
                        timestamp: event.commit_timestamp,
                    };

                    // Track by file path
                    file_changes.entry(file_change_info.path.clone())
                        .or_insert_with(Vec::new)
                        .push(change.clone());

                    // Track renames
                    if let FileChangeType::Renamed = &file_change_info.change_type {
                        // In a real implementation, we'd need the old path from the event
                        // For now, we'll just note that this is a renamed file
                        let rename = RenameInfo {
                            old_path: file_change_info.path.clone(), // Would need old path
                            new_path: file_change_info.path.clone(),
                            commit_hash: event.commit_hash.clone(),
                            timestamp: event.commit_timestamp,
                        };
                        rename_history.entry(file_change_info.path.clone())
                            .or_insert_with(Vec::new)
                            .push(rename);
                    }

                    changes_for_commit.push(change);
                }

                // Track by commit
                commit_changes.insert(event.commit_hash.clone(), changes_for_commit);
            }
            _ => {} // Other events don't affect file changes
        }

        Ok(())
    }

    /// Get file history for a specific path
    pub fn get_file_history(&self, path: &FilePath) -> Result<Vec<FileChange>, ProjectionError> {
        let file_changes = self.file_changes.read()
            .map_err(|_| ProjectionError::LockError)?;

        Ok(file_changes.get(path)
            .cloned()
            .unwrap_or_default())
    }

    /// Get all changes in a specific commit
    pub fn get_commit_changes(&self, commit_hash: &CommitHash) -> Result<Vec<FileChange>, ProjectionError> {
        let commit_changes = self.commit_changes.read()
            .map_err(|_| ProjectionError::LockError)?;

        Ok(commit_changes.get(commit_hash)
            .cloned()
            .unwrap_or_default())
    }

    /// Get files changed between two commits
    pub fn get_changes_between(
        &self,
        _from_commit: &CommitHash,
        to_commit: &CommitHash,
    ) -> Result<Vec<FileChange>, ProjectionError> {
        let commit_changes = self.commit_changes.read()
            .map_err(|_| ProjectionError::LockError)?;

        // In a real implementation, we'd need to walk the commit graph
        // For now, return changes from the to_commit
        Ok(commit_changes.get(to_commit)
            .cloned()
            .unwrap_or_default())
    }

    /// Get rename history for a file
    pub fn get_rename_history(&self, path: &FilePath) -> Result<Vec<RenameInfo>, ProjectionError> {
        let rename_history = self.rename_history.read()
            .map_err(|_| ProjectionError::LockError)?;

        Ok(rename_history.get(path)
            .cloned()
            .unwrap_or_default())
    }

    /// Get statistics for file changes
    pub fn get_file_statistics(&self, path: &FilePath) -> Result<FileStatistics, ProjectionError> {
        let changes = self.get_file_history(path)?;

        let total_additions: usize = changes.iter().map(|c| c.additions).sum();
        let total_deletions: usize = changes.iter().map(|c| c.deletions).sum();
        let change_count = changes.len();

        let authors: HashSet<_> = changes.iter()
            .map(|c| c.author.name.clone())
            .collect();

        Ok(FileStatistics {
            path: path.clone(),
            total_additions,
            total_deletions,
            change_count,
            unique_authors: authors.len(),
            first_commit: changes.first().map(|c| c.commit_hash.clone()),
            last_commit: changes.last().map(|c| c.commit_hash.clone()),
        })
    }
}

/// File statistics summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileStatistics {
    /// File path
    pub path: FilePath,
    /// Total lines added
    pub total_additions: usize,
    /// Total lines deleted
    pub total_deletions: usize,
    /// Number of changes
    pub change_count: usize,
    /// Number of unique authors
    pub unique_authors: usize,
    /// First commit touching this file
    pub first_commit: Option<CommitHash>,
    /// Last commit touching this file
    pub last_commit: Option<CommitHash>,
}

impl Default for FileChangeProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for projection operations
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    /// Lock was poisoned
    #[error("Lock was poisoned")]
    LockPoisoned,
    
    /// Lock error
    #[error("Lock error")]
    LockError,
    
    /// Other projection error
    #[error("Projection error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{RepositoryAnalyzed, BranchCreated, CommitAnalyzed, FileChangeInfo};
    use crate::value_objects::{AuthorInfo, FilePath};

    #[test]
    fn test_repository_list_projection() {
        let projection = RepositoryListProjection::new();
        let repo_id = RepositoryId::new();
        
        // Handle repository analyzed event
        let event = GitDomainEvent::RepositoryAnalyzed(RepositoryAnalyzed {
            repository_id: repo_id,
            path: "/tmp/test-repo".to_string(),
            name: "test-repo".to_string(),
            branch_count: 2,
            commit_count: 10,
            timestamp: Utc::now(),
        });
        
        projection.handle_event(&event).unwrap();
        
        // Check projection state
        let repos = projection.get_all().unwrap();
        assert_eq!(repos.len(), 1);
        
        let summary = &repos[0];
        assert_eq!(summary.name, "test-repo");
        assert_eq!(summary.branch_count, 2);
        assert_eq!(summary.commit_count, 10);
    }

    #[test]
    fn test_commit_history_projection() {
        let projection = CommitHistoryProjection::new();
        let repo_id = RepositoryId::new();
        
        // Handle commit analyzed event
        let event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
            repository_id: repo_id,
            commit_hash: CommitHash::new("abc123def").unwrap(),
            parents: vec![],
            author: AuthorInfo::new("Test Author", "test@example.com"),
            message: "Test commit".to_string(),
            files_changed: vec![],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });
        
        projection.handle_event(&event).unwrap();
        
        // Check projection state
        let history = projection.get_history(&repo_id, None).unwrap();
        assert_eq!(history.len(), 1);
        
        let commit = &history[0];
        assert_eq!(commit.message, "Test commit");
        assert_eq!(commit.author_name, "Test Author");
    }

    #[test]
    fn test_branch_status_projection() {
        let projection = BranchStatusProjection::new();
        let repo_id = RepositoryId::new();
        
        // Handle branch created event
        let event = GitDomainEvent::BranchCreated(BranchCreated {
            repository_id: repo_id,
            branch_name: BranchName::new("main").unwrap(),
            commit_hash: CommitHash::new("abc123def").unwrap(),
            source_branch: None,
            timestamp: Utc::now(),
        });
        
        projection.handle_event(&event).unwrap();
        
        // Check projection state
        let branches = projection.get_branches(&repo_id).unwrap();
        assert_eq!(branches.len(), 1);
        
        let branch = &branches[0];
        assert_eq!(branch.name.as_str(), "main");
        assert!(branch.is_default);
    }

    #[tokio::test]
    async fn test_file_change_projection() {
        let projection = FileChangeProjection::new();

        // Create a commit with file changes
        let file_path = FilePath::new("src/main.rs").unwrap();
        let event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
            repository_id: RepositoryId::new(),
            commit_hash: CommitHash::new("abc123def456789").unwrap(),
            parents: vec![],
            author: AuthorInfo::new("Test Author".to_string(), "test@example.com".to_string()),
            message: "Test commit".to_string(),
            files_changed: vec![
                FileChangeInfo {
                    path: file_path.clone(),
                    additions: 10,
                    deletions: 5,
                    change_type: FileChangeType::Modified,
                },
            ],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });

        projection.handle_event(&event).await.unwrap();

        // Get file history
        let history = projection.get_file_history(&file_path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].additions, 10);
        assert_eq!(history[0].deletions, 5);

        // Get file statistics
        let stats = projection.get_file_statistics(&file_path).unwrap();
        assert_eq!(stats.total_additions, 10);
        assert_eq!(stats.total_deletions, 5);
        assert_eq!(stats.change_count, 1);
        assert_eq!(stats.unique_authors, 1);
    }
}
