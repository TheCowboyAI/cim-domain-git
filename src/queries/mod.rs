//! Query handlers for the Git domain
//!
//! This module contains query handlers that read from projections
//! to answer queries about the Git domain.

use crate::aggregate::RepositoryId;
use crate::projections::{
    RepositoryListProjection, RepositorySummary,
    CommitHistoryProjection, CommitHistoryEntry,
    BranchStatusProjection, BranchInfo,
};
use crate::value_objects::BranchName;
use cim_domain::Query;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query to get repository details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryDetails {
    /// Repository ID to get details for
    pub repository_id: RepositoryId,
}

impl Query for GetRepositoryDetails {}

/// Result of repository details query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDetailsResult {
    /// Repository summary
    pub summary: Option<RepositorySummary>,
    /// Recent commits
    pub recent_commits: Vec<CommitHistoryEntry>,
    /// Branches
    pub branches: Vec<BranchInfo>,
}

/// Query to get commit history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCommitHistory {
    /// Repository ID
    pub repository_id: RepositoryId,
    /// Maximum number of commits to return
    pub limit: Option<usize>,
}

impl Query for GetCommitHistory {}

/// Result of commit history query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitHistoryResult {
    /// Commit history entries
    pub commits: Vec<CommitHistoryEntry>,
    /// Total number of commits (before limit)
    pub total_count: usize,
}

/// Query to get branch list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBranchList {
    /// Repository ID
    pub repository_id: RepositoryId,
}

impl Query for GetBranchList {}

/// Result of branch list query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchListResult {
    /// List of branches
    pub branches: Vec<BranchInfo>,
    /// Default branch name if any
    pub default_branch: Option<BranchName>,
}

/// Query to list all repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    /// Optional filter by remote URL pattern
    pub remote_url_pattern: Option<String>,
}

impl Query for ListRepositories {}

/// Result of list repositories query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoriesResult {
    /// List of repository summaries
    pub repositories: Vec<RepositorySummary>,
}

/// Query handler for Git domain queries
pub struct GitQueryHandler {
    repository_projection: Arc<RepositoryListProjection>,
    commit_projection: Arc<CommitHistoryProjection>,
    branch_projection: Arc<BranchStatusProjection>,
}

impl GitQueryHandler {
    /// Create a new query handler
    pub fn new(
        repository_projection: Arc<RepositoryListProjection>,
        commit_projection: Arc<CommitHistoryProjection>,
        branch_projection: Arc<BranchStatusProjection>,
    ) -> Self {
        Self {
            repository_projection,
            commit_projection,
            branch_projection,
        }
    }

    /// Handle GetRepositoryDetails query
    pub async fn handle_get_repository_details(
        &self,
        query: GetRepositoryDetails,
    ) -> Result<RepositoryDetailsResult, QueryError> {
        // Get repository summary
        let summary = self.repository_projection
            .get_by_id(&query.repository_id)
            .map_err(|e| QueryError::ProjectionError(e.to_string()))?;

        // Get recent commits (limit to 10)
        let recent_commits = self.commit_projection
            .get_history(&query.repository_id, Some(10))
            .map_err(|e| QueryError::ProjectionError(e.to_string()))?;

        // Get branches
        let branches = self.branch_projection
            .get_branches(&query.repository_id)
            .map_err(|e| QueryError::ProjectionError(e.to_string()))?;

        Ok(RepositoryDetailsResult {
            summary,
            recent_commits,
            branches,
        })
    }

    /// Handle GetCommitHistory query
    pub async fn handle_get_commit_history(
        &self,
        query: GetCommitHistory,
    ) -> Result<CommitHistoryResult, QueryError> {
        // Get full history to count
        let all_commits = self.commit_projection
            .get_history(&query.repository_id, None)
            .map_err(|e| QueryError::ProjectionError(e.to_string()))?;
        
        let total_count = all_commits.len();

        // Get limited history if requested
        let commits = if let Some(limit) = query.limit {
            self.commit_projection
                .get_history(&query.repository_id, Some(limit))
                .map_err(|e| QueryError::ProjectionError(e.to_string()))?
        } else {
            all_commits
        };

        Ok(CommitHistoryResult {
            commits,
            total_count,
        })
    }

    /// Handle GetBranchList query
    pub async fn handle_get_branch_list(
        &self,
        query: GetBranchList,
    ) -> Result<BranchListResult, QueryError> {
        let branches = self.branch_projection
            .get_branches(&query.repository_id)
            .map_err(|e| QueryError::ProjectionError(e.to_string()))?;

        // Find default branch
        let default_branch = branches.iter()
            .find(|b| b.is_default)
            .map(|b| b.name.clone());

        Ok(BranchListResult {
            branches,
            default_branch,
        })
    }

    /// Handle ListRepositories query
    pub async fn handle_list_repositories(
        &self,
        query: ListRepositories,
    ) -> Result<ListRepositoriesResult, QueryError> {
        let repositories = if let Some(pattern) = query.remote_url_pattern {
            self.repository_projection
                .find_by_remote_url(&pattern)
                .map_err(|e| QueryError::ProjectionError(e.to_string()))?
        } else {
            self.repository_projection
                .get_all()
                .map_err(|e| QueryError::ProjectionError(e.to_string()))?
        };

        Ok(ListRepositoriesResult { repositories })
    }


}

/// Error type for query operations
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// Projection error
    #[error("Projection error: {0}")]
    ProjectionError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Unknown query type
    #[error("Unknown query type: {0}")]
    UnknownQueryType(String),

    /// Other error
    #[error("Query error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{GitDomainEvent, RepositoryAnalyzed, CommitAnalyzed, BranchCreated};
    use crate::value_objects::{AuthorInfo, CommitHash};
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_repository_details() {
        // Create projections
        let repo_projection = Arc::new(RepositoryListProjection::new());
        let commit_projection = Arc::new(CommitHistoryProjection::new());
        let branch_projection = Arc::new(BranchStatusProjection::new());

        // Create query handler
        let handler = GitQueryHandler::new(
            repo_projection.clone(),
            commit_projection.clone(),
            branch_projection.clone(),
        );

        // Setup test data
        let repo_id = RepositoryId::new();

        // Add repository
        let repo_event = GitDomainEvent::RepositoryAnalyzed(RepositoryAnalyzed {
            repository_id: repo_id,
            path: "/tmp/test-repo".to_string(),
            name: "test-repo".to_string(),
            branch_count: 1,
            commit_count: 1,
            timestamp: Utc::now(),
        });
        repo_projection.handle_event(&repo_event).unwrap();

        // Add commit
        let commit_event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
            repository_id: repo_id,
            commit_hash: CommitHash::new("abc123def").unwrap(),
            parents: vec![],
            author: AuthorInfo::new("Test Author", "test@example.com"),
            message: "Initial commit".to_string(),
            files_changed: vec![],
            commit_timestamp: Utc::now(),
            timestamp: Utc::now(),
        });
        commit_projection.handle_event(&commit_event).unwrap();

        // Add branch
        let branch_event = GitDomainEvent::BranchCreated(BranchCreated {
            repository_id: repo_id,
            branch_name: BranchName::new("main").unwrap(),
            commit_hash: CommitHash::new("abc123def").unwrap(),
            source_branch: None,
            timestamp: Utc::now(),
        });
        branch_projection.handle_event(&branch_event).unwrap();

        // Query repository details
        let query = GetRepositoryDetails { repository_id: repo_id };
        let result = handler.handle_get_repository_details(query).await.unwrap();

        // Verify results
        assert!(result.summary.is_some());
        assert_eq!(result.summary.unwrap().name, "test-repo");
        assert_eq!(result.recent_commits.len(), 1);
        assert_eq!(result.branches.len(), 1);
    }

    #[tokio::test]
    async fn test_list_repositories() {
        // Create projections
        let repo_projection = Arc::new(RepositoryListProjection::new());
        let commit_projection = Arc::new(CommitHistoryProjection::new());
        let branch_projection = Arc::new(BranchStatusProjection::new());

        // Create query handler
        let handler = GitQueryHandler::new(
            repo_projection.clone(),
            commit_projection,
            branch_projection,
        );

        // Add test repositories
        for i in 0..3 {
            let event = GitDomainEvent::RepositoryAnalyzed(RepositoryAnalyzed {
                repository_id: RepositoryId::new(),
                path: format!("/tmp/repo-{}", i),
                name: format!("repo-{}", i),
                branch_count: 1,
                commit_count: 10,
                timestamp: Utc::now(),
            });
            repo_projection.handle_event(&event).unwrap();
        }

        // Query all repositories
        let query = ListRepositories { remote_url_pattern: None };
        let result = handler.handle_list_repositories(query).await.unwrap();

        // Verify results
        assert_eq!(result.repositories.len(), 3);
    }
}
