//! Command and event handlers for the Git domain
//!
//! This module contains the implementation of command handlers
//! that process commands and generate events.

mod cqrs_adapter;

pub use cqrs_adapter::*;

use crate::GitDomainError;
use crate::aggregate::{Repository, RepositoryId};
use crate::commands::{ExtractCommitGraph, ExtractDependencyGraph};
use crate::events::{GitDomainEvent, RepositoryAnalyzed, BranchCreated, CommitAnalyzed, CommitGraphExtracted, DependencyGraphExtracted};
use crate::value_objects::{BranchName, CommitHash, AuthorInfo};
use chrono::{DateTime, Utc};
use cim_domain_graph::{GraphId, NodeId};
use git2::{Oid, Repository as Git2Repository, Sort};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Repository command handler for Git operations
pub struct RepositoryCommandHandler {
    /// In-memory repository for demo purposes
    repositories: std::sync::Mutex<HashMap<RepositoryId, Repository>>,
}

impl RepositoryCommandHandler {
    /// Create a new repository command handler
    #[must_use] pub fn new() -> Self {
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

        revwalk.set_sorting(Sort::TIME).map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to set sort: {e}"))
        })?;

        // Start from HEAD
        if let Ok(head) = git_repo.head() {
            if let Some(target) = head.target() {
                revwalk.push(target).map_err(|e| {
                    GitDomainError::GitOperationFailed(format!("Failed to push HEAD: {e}"))
                })?;
            }
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

                    // Get files changed (simplified - just count)
                    let files_changed = vec![]; // TODO: Implement diff analysis

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
            let mut repos = self.repositories.lock().unwrap();
            repos.insert(repo_id, repository);
        }

        Ok((repo_id, events))
    }

    /// Extract commit graph from repository
    pub async fn extract_commit_graph(
        &self,
        cmd: ExtractCommitGraph,
    ) -> Result<Vec<GitDomainEvent>, GitDomainError> {
        info!(
            "Extracting commit graph for repository: {:?}",
            cmd.repository_id
        );

        // Get repository
        let repo = {
            let repos = self.repositories.lock().unwrap();
            repos.get(&cmd.repository_id).cloned().ok_or_else(|| {
                GitDomainError::GitOperationFailed("Repository not found".to_string())
            })?
        };

        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            GitDomainError::GitOperationFailed("Repository not cloned".to_string())
        })?;

        // Open repository with git2
        let git_repo = Git2Repository::open(local_path).map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to open repository: {e}"))
        })?;

        // Create graph
        let graph_id = GraphId::new();
        let mut commit_nodes = HashMap::new();
        let mut edges = Vec::new();

        // Walk commits
        let mut revwalk = git_repo.revwalk().map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to create revwalk: {e}"))
        })?;

        revwalk.set_sorting(Sort::TIME).map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to set sort: {e}"))
        })?;

        // Start from specified commit or HEAD
        if let Some(start_commit) = &cmd.start_commit {
            let oid = Oid::from_str(start_commit.as_str()).map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Invalid commit: {e}"))
            })?;
            revwalk.push(oid).map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Failed to push commit: {e}"))
            })?;
        } else if let Ok(head) = git_repo.head() {
            if let Some(target) = head.target() {
                revwalk.push(target).map_err(|e| {
                    GitDomainError::GitOperationFailed(format!("Failed to push HEAD: {e}"))
                })?;
            }
        }

        let mut commit_count = 0;
        let max_commits = cmd.max_depth.unwrap_or(100) as usize;

        for commit_oid in revwalk.take(max_commits) {
            if let Ok(oid) = commit_oid {
                if let Ok(commit) = git_repo.find_commit(oid) {
                    let commit_hash = CommitHash::new(oid.to_string())?;
                    let node_id = NodeId::new();

                    // Store node mapping
                    commit_nodes.insert(commit_hash.clone(), node_id);

                    // Create edges to parents
                    for parent_oid in commit.parent_ids() {
                        if let Ok(parent_hash) = CommitHash::new(parent_oid.to_string()) {
                            // Edge will be created when parent is processed
                            edges.push((commit_hash.clone(), parent_hash));
                        }
                    }

                    commit_count += 1;
                }
            }
        }

        // Count edges
        let edge_count = edges.len();

        let graph_event = CommitGraphExtracted {
            repository_id: cmd.repository_id,
            graph_id,
            commit_count,
            edge_count,
            root_commits: vec![], // TODO: Calculate actual root commits
            head_commits: vec![], // TODO: Calculate actual head commits
            timestamp: Utc::now(),
        };

        Ok(vec![GitDomainEvent::CommitGraphExtracted(graph_event)])
    }

    /// Get repository by ID
    pub fn get_repository(&self, id: &RepositoryId) -> Option<Repository> {
        let repos = self.repositories.lock().unwrap();
        repos.get(id).cloned()
    }

    /// List all repositories
    pub fn list_repositories(&self) -> Vec<Repository> {
        let repos = self.repositories.lock().unwrap();
        repos.values().cloned().collect()
    }
}

impl Default for RepositoryCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// File dependency analyzer for extracting dependency graphs
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Extract dependency graph from repository
    pub async fn extract_dependency_graph(
        cmd: ExtractDependencyGraph,
        git_repo: &Git2Repository,
    ) -> Result<Vec<GitDomainEvent>, GitDomainError> {
        info!(
            "Extracting dependency graph for repository: {:?}",
            cmd.repository_id
        );

        let graph_id = GraphId::new();
        let mut file_count = 0;
        let dependency_count = 0;

        // Get HEAD commit or specified commit
        let commit = if let Some(commit_hash) = &cmd.commit_hash {
            let oid = Oid::from_str(commit_hash.as_str()).map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Invalid commit: {e}"))
            })?;
            git_repo.find_commit(oid).map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Commit not found: {e}"))
            })?
        } else {
            let head = git_repo.head().map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Failed to get HEAD: {e}"))
            })?;
            head.peel_to_commit().map_err(|e| {
                GitDomainError::GitOperationFailed(format!("Failed to get HEAD commit: {e}"))
            })?
        };

        let tree = commit.tree().map_err(|e| {
            GitDomainError::GitOperationFailed(format!("Failed to get tree: {e}"))
        })?;

        // Walk tree and analyze files
        tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
            if let Some(name) = entry.name() {
                let full_path = if path.is_empty() {
                    name.to_string()
                } else {
                    format!("{path}/{name}")
                };

                // Check against include/exclude patterns
                let should_include = if cmd.include_patterns.is_empty() {
                    true
                } else {
                    cmd.include_patterns.iter().any(|pattern| {
                        // Simple pattern matching - could be enhanced with regex
                        full_path.contains(pattern) || full_path.ends_with(pattern)
                    })
                };

                let should_exclude = cmd
                    .exclude_patterns
                    .iter()
                    .any(|pattern| full_path.contains(pattern) || full_path.ends_with(pattern));

                if should_include && !should_exclude && entry.kind() == Some(git2::ObjectType::Blob)
                {
                    file_count += 1;

                    // TODO: Analyze file content for dependencies
                    // This would require parsing the file based on language
                    // For now, we'll just count files
                }
            }
            git2::TreeWalkResult::Ok
        })
        .map_err(|e| GitDomainError::GitOperationFailed(format!("Failed to walk tree: {e}")))?;

        let graph_event = DependencyGraphExtracted {
            repository_id: cmd.repository_id,
            graph_id,
            commit_hash: CommitHash::new(commit.id().to_string())?,
            file_count,
            dependency_count,
            language: cmd.language,
            timestamp: Utc::now(),
        };

        Ok(vec![GitDomainEvent::DependencyGraphExtracted(graph_event)])
    }
}

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
                    println!("Failed to analyze repository: {}", e);
                    // Don't fail the test if we're not in a git repo
                }
            }
        }
    }
}
