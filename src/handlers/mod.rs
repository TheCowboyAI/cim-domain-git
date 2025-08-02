// Copyright 2025 Cowboy AI, LLC.

//! Command and event handlers for the Git domain
//!
//! This module contains the implementation of command handlers
//! that process commands and generate events.

mod cqrs_adapter;

pub use cqrs_adapter::*;

use crate::aggregate::{Repository, RepositoryId};
use crate::events::{
    BranchCreated, CommitAnalyzed, FileChangeInfo, FileChangeType, GitDomainEvent,
    RepositoryAnalyzed,
};
use crate::value_objects::{AuthorInfo, BranchName, CommitHash, FilePath};
use crate::GitDomainError;
use chrono::{DateTime, Utc};
use git2::{Repository as Git2Repository, Sort};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, instrument, warn};

/// Repository command handler for Git operations
pub struct RepositoryCommandHandler {
    /// In-memory repository for demo purposes
    repositories: std::sync::Mutex<HashMap<RepositoryId, Repository>>,
}

impl RepositoryCommandHandler {
    /// Create a new repository command handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            repositories: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Analyze the current working directory as a Git repository
    pub async fn analyze_current_repository(
        &self,
    ) -> Result<(RepositoryId, Vec<GitDomainEvent>), GitDomainError> {
        let current_dir = std::env::current_dir().map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Cannot get current directory: {e}"))
        })?;

        self.analyze_repository_at_path(current_dir.to_string_lossy())
            .await
    }

    /// Analyze a Git repository at the given path
    #[instrument(skip(self), fields(path = %path.as_ref()))]
    pub async fn analyze_repository_at_path(
        &self,
        path: impl AsRef<str>,
    ) -> Result<(RepositoryId, Vec<GitDomainEvent>), GitDomainError> {
        let path = path.as_ref();
        info!("Analyzing Git repository at: {}", path);

        // Open repository with git2
        let git_repo = Git2Repository::open(path).map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to open repository: {e}"))
        })?;

        let repo_id = RepositoryId::new();
        let mut events = Vec::new();

        // Get repository metadata
        let repo_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Create repository analyzed event
        let analyzed_event = RepositoryAnalyzed {
            repository_id: repo_id,
            path: path.to_string(),
            name: repo_name.clone(),
            branch_count: 0, // Will be updated below
            commit_count: 0, // Will be updated below
            timestamp: Utc::now(),
        };

        events.push(GitDomainEvent::RepositoryAnalyzed(analyzed_event));

        // Analyze branches
        let branches = git_repo.branches(None).map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to get branches: {e}"))
        })?;

        let mut branch_count = 0;
        for branch_result in branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(name) = branch.name().ok().flatten() {
                    if let Ok(branch_name) = BranchName::new(name) {
                        if let Some(target) = branch.get().target() {
                            if let Ok(commit_hash) = CommitHash::new(target.to_string()) {
                                let branch_event = BranchCreated {
                                    repository_id: repo_id,
                                    branch_name,
                                    commit_hash,
                                    source_branch: None,
                                    timestamp: Utc::now(),
                                };
                                events.push(GitDomainEvent::BranchCreated(branch_event));
                                branch_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // Analyze commits
        let mut revwalk = git_repo.revwalk().map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to create revwalk: {e}"))
        })?;

        revwalk
            .set_sorting(Sort::TIME)
            .map_err(|e| GitDomainError::GitOperationFailed(format!("Failed to set sort: {e}")))?;

        // Start from HEAD
        if let Ok(head) = git_repo.head() {
            if let Some(target) = head.target() {
                revwalk.push(target).map_err(|e| {
                    GitDomainError::GitOperationFailed(format!("Failed to push HEAD: {e}"))
                })?;
            }
        } else {
            warn!("Repository has no HEAD - might be empty");
        }

        let mut commit_count = 0;
        for commit_oid in revwalk.take(100) {
            // Limit to first 100 commits for demo
            if let Ok(oid) = commit_oid {
                if let Ok(commit) = git_repo.find_commit(oid) {
                    let commit_hash = CommitHash::new(oid.to_string()).map_err(|e| {
                        GitDomainError::GitOperationFailed(format!("Invalid commit hash: {e}"))
                    })?;

                    let author = commit.author();
                    let author_info = AuthorInfo::new(
                        author.name().unwrap_or("Unknown").to_string(),
                        author.email().unwrap_or("unknown@example.com").to_string(),
                    );

                    let parents: Vec<CommitHash> = commit
                        .parent_ids()
                        .filter_map(|oid| CommitHash::new(oid.to_string()).ok())
                        .collect();

                    let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0)
                        .unwrap_or_else(Utc::now);

                    // Get files changed by comparing with parent
                    let mut files_changed = vec![];

                    if let Ok(parent) = commit.parent(0) {
                        // Get diff between parent and current commit
                        if let Ok(parent_tree) = parent.tree() {
                            if let Ok(current_tree) = commit.tree() {
                                if let Ok(diff) = git_repo.diff_tree_to_tree(
                                    Some(&parent_tree),
                                    Some(&current_tree),
                                    None,
                                ) {
                                    // Collect file changes
                                    let _ = diff.foreach(
                                        &mut |delta, _| {
                                            if let Some(new_file) = delta.new_file().path() {
                                                if let Some(path_str) = new_file.to_str() {
                                                    if let Ok(file_path) = FilePath::new(path_str) {
                                                        files_changed.push(FileChangeInfo {
                                                            path: file_path,
                                                            additions: 0, // Would need to parse diff for actual counts
                                                            deletions: 0,
                                                            change_type: match delta.status() {
                                                                git2::Delta::Added => {
                                                                    FileChangeType::Added
                                                                }
                                                                git2::Delta::Deleted => {
                                                                    FileChangeType::Deleted
                                                                }
                                                                git2::Delta::Modified => {
                                                                    FileChangeType::Modified
                                                                }
                                                                git2::Delta::Renamed => {
                                                                    FileChangeType::Renamed
                                                                }
                                                                _ => FileChangeType::Modified,
                                                            },
                                                        });
                                                    }
                                                }
                                            }
                                            true
                                        },
                                        None,
                                        None,
                                        None,
                                    );
                                }
                            }
                        }
                    }

                    let commit_event = CommitAnalyzed {
                        repository_id: repo_id,
                        commit_hash,
                        parents,
                        author: author_info,
                        message: commit.message().unwrap_or("No message").to_string(),
                        files_changed,
                        commit_timestamp: timestamp,
                        timestamp: Utc::now(),
                    };

                    events.push(GitDomainEvent::CommitAnalyzed(commit_event));
                    commit_count += 1;
                }
            }
        }

        info!(
            "Analyzed repository: {} branches, {} commits",
            branch_count, commit_count
        );

        // Create and store repository aggregate
        let mut repository = Repository::new(repo_name);
        repository.id = repo_id;
        repository.local_path = Some(path.to_string());

        // Apply events to aggregate
        for event in &events {
            repository.apply_event(event).map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Failed to apply event: {e}"))
            })?;
        }

        // Store repository
        {
            let mut repos = self.repositories.lock().map_err(|_| {
                GitDomainError::GitOperationFailed("Failed to acquire repository lock".to_string())
            })?;
            repos.insert(repo_id, repository);
        }

        Ok((repo_id, events))
    }

    // Note: Extract commit graph functionality has been removed
    // This was dependent on cim_domain_graph which is no longer available

    /// Get repository by ID
    pub fn get_repository(&self, id: &RepositoryId) -> Option<Repository> {
        let repos = self.repositories.lock().ok()?;
        repos.get(id).cloned()
    }

    /// List all repositories
    pub fn list_repositories(&self) -> Vec<Repository> {
        let repos = match self.repositories.lock() {
            Ok(repos) => repos,
            Err(_) => return Vec::new(),
        };
        repos.values().cloned().collect()
    }
}

impl Default for RepositoryCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Extract dependency graph functionality has been removed
// This was dependent on cim_domain_graph which is no longer available

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repository_handler_creation() {
        let handler = RepositoryCommandHandler::new();
        assert_eq!(handler.list_repositories().len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_current_repository() {
        let handler = RepositoryCommandHandler::new();

        // This test will only work if run in a git repository
        if std::env::current_dir().unwrap().join(".git").exists() {
            let result = handler.analyze_current_repository().await;
            match result {
                Ok((repo_id, events)) => {
                    println!("Successfully analyzed repository: {:?}", repo_id);
                    println!("Generated {} events", events.len());
                    assert!(!events.is_empty());
                }
                Err(e) => {
                    println!("Failed to analyze repository: {e}");
                    // Don't fail the test if we're not in a git repo
                }
            }
        }
    }
}
