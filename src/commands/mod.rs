//! Commands for the Git domain
//!
//! Commands represent intentions to change the state of the Git domain.
//! They are validated and processed by command handlers.

use crate::aggregate::RepositoryId;
use crate::value_objects::{BranchName, CommitHash, FilePath, RemoteUrl, TagName};
use serde::{Deserialize, Serialize};

/// Clone a repository from a remote URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneRepository {
    /// Repository ID (if creating new) or existing ID
    pub repository_id: Option<RepositoryId>,

    /// Remote URL to clone from
    pub remote_url: RemoteUrl,

    /// Local path to clone to
    pub local_path: String,

    /// Branch to checkout (defaults to default branch)
    pub branch: Option<BranchName>,

    /// Depth of clone (for shallow clones)
    pub depth: Option<u32>,
}

/// Analyze a specific commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCommit {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Commit hash to analyze
    pub commit_hash: CommitHash,

    /// Whether to analyze file contents
    pub analyze_files: bool,

    /// Whether to extract dependencies
    pub extract_dependencies: bool,
}

/// Extract the commit graph from a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractCommitGraph {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Starting commit (defaults to HEAD)
    pub start_commit: Option<CommitHash>,

    /// Maximum depth to traverse
    pub max_depth: Option<u32>,

    /// Whether to include all branches
    pub include_all_branches: bool,

    /// Whether to include tags
    pub include_tags: bool,
}

/// Extract file dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractDependencyGraph {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Commit to analyze (defaults to HEAD)
    pub commit_hash: Option<CommitHash>,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Programming language to analyze
    pub language: Option<String>,
}

/// Create a new branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranch {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Branch name
    pub branch_name: BranchName,

    /// Starting point (commit or branch)
    pub start_point: String,

    /// Whether to checkout the new branch
    pub checkout: bool,
}

/// Delete a branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBranch {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Branch name to delete
    pub branch_name: BranchName,

    /// Force deletion even if not merged
    pub force: bool,
}

/// Create a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTag {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Tag name
    pub tag_name: TagName,

    /// Commit to tag (defaults to HEAD)
    pub commit_hash: Option<CommitHash>,

    /// Tag message (for annotated tags)
    pub message: Option<String>,

    /// Whether to create annotated tag
    pub annotated: bool,
}

/// Analyze repository structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRepository {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Whether to update metadata
    pub update_metadata: bool,

    /// Whether to analyze languages
    pub analyze_languages: bool,

    /// Whether to calculate statistics
    pub calculate_statistics: bool,
}

/// Fetch updates from remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRemote {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Remote name (defaults to origin)
    pub remote: Option<String>,

    /// Whether to fetch all remotes
    pub all_remotes: bool,

    /// Whether to prune deleted branches
    pub prune: bool,
}

/// Analyze file history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeFileHistory {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// File path to analyze
    pub file_path: FilePath,

    /// Starting commit
    pub start_commit: Option<CommitHash>,

    /// Ending commit
    pub end_commit: Option<CommitHash>,

    /// Whether to follow renames
    pub follow_renames: bool,
}

/// Compare branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBranches {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Base branch
    pub base_branch: BranchName,

    /// Compare branch
    pub compare_branch: BranchName,

    /// Whether to include file diffs
    pub include_diffs: bool,
}

/// Search repository content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRepository {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Search pattern (regex)
    pub pattern: String,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Whether search is case sensitive
    pub case_sensitive: bool,

    /// Maximum results to return
    pub max_results: Option<usize>,
}

/// Integrate with GitHub via MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIntegration {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// GitHub repository (owner/name)
    pub github_repo: String,

    /// Operations to perform
    pub operations: Vec<GitHubOperation>,
}

/// GitHub operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitHubOperation {
    /// Sync issues
    SyncIssues,
    /// Sync pull requests
    SyncPullRequests,
    /// Sync releases
    SyncReleases,
    /// Sync workflows
    SyncWorkflows,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = CloneRepository {
            repository_id: None,
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            branch: None,
            depth: None,
        };

        assert_eq!(cmd.local_path, "/tmp/repo");
        assert!(cmd.repository_id.is_none());
    }
}
