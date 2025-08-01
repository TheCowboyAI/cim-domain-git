// Copyright 2025 Cowboy AI, LLC.

//! Commands for the Git domain
//!
//! Commands represent intentions to change the state of the Git domain.
//! They are validated and processed by command handlers.

use crate::aggregate::{Repository, RepositoryId};
use crate::value_objects::{BranchName, CommitHash, FilePath, RemoteUrl, TagName};
use cim_domain::{Command, EntityId};
use serde::{Deserialize, Serialize};

/// Clone a repository from a remote URL
///
/// This command initiates the cloning of a Git repository from a remote
/// location to a local path. It supports shallow cloning and branch selection.
///
/// # Examples
///
/// ```
/// use cim_domain_git::commands::CloneRepository;
/// use cim_domain_git::value_objects::{RemoteUrl, BranchName};
/// use cim_domain_git::aggregate::RepositoryId;
///
/// // Clone a new repository
/// let clone_cmd = CloneRepository {
///     repository_id: None, // Will create new repository
///     remote_url: RemoteUrl::new("https://github.com/rust-lang/rust.git").unwrap(),
///     local_path: "/workspace/rust".to_string(),
///     branch: None, // Use default branch
///     depth: None, // Full clone
/// };
///
/// // Clone with specific branch and shallow depth
/// let shallow_clone = CloneRepository {
///     repository_id: Some(RepositoryId::new()),
///     remote_url: RemoteUrl::new("https://github.com/tokio-rs/tokio.git").unwrap(),
///     local_path: "/tmp/tokio".to_string(),
///     branch: Some(BranchName::new("v1.0.0").unwrap()),
///     depth: Some(1), // Only latest commit
/// };
///
/// assert_eq!(shallow_clone.local_path, "/tmp/tokio");
/// ```
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

impl Command for CloneRepository {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        self.repository_id.map(|id| EntityId::from_uuid(*id.as_uuid()))
    }
}

/// Analyze a specific commit
///
/// This command triggers analysis of a specific commit, including
/// file changes, dependencies, and other metadata extraction.
///
/// # Examples
///
/// ```
/// use cim_domain_git::commands::AnalyzeCommit;
/// use cim_domain_git::aggregate::RepositoryId;
/// use cim_domain_git::value_objects::CommitHash;
/// use cim_domain::Command;
///
/// let repo_id = RepositoryId::new();
/// let commit_hash = CommitHash::new("abc123def456789").unwrap();
///
/// // Basic commit analysis
/// let analyze_cmd = AnalyzeCommit {
///     repository_id: repo_id,
///     commit_hash: commit_hash.clone(),
///     analyze_files: false,
///     extract_dependencies: false,
/// };
///
/// // Full analysis with file and dependency extraction
/// let full_analysis = AnalyzeCommit {
///     repository_id: repo_id,
///     commit_hash,
///     analyze_files: true,
///     extract_dependencies: true,
/// };
///
/// // Verify aggregate ID is set correctly
/// assert!(full_analysis.aggregate_id().is_some());
/// ```
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

impl Command for AnalyzeCommit {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
}

// TODO: ExtractCommitGraph has been removed
// This was dependent on cim_domain_graph which is no longer available
//
// /// Extract the commit graph from a repository
// ///
// /// This command extracts the commit graph structure, showing the relationships
// /// between commits, branches, and merge points.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExtractCommitGraph {
//     /// Repository ID
//     pub repository_id: RepositoryId,
//
//     /// Starting commit (defaults to HEAD)
//     pub start_commit: Option<CommitHash>,
//
//     /// Maximum depth to traverse
//     pub max_depth: Option<u32>,
//
//     /// Whether to include all branches
//     pub include_all_branches: bool,
//
//     /// Whether to include tags
//     pub include_tags: bool,
// }
//
// impl Command for ExtractCommitGraph {
//     type Aggregate = Repository;
//
//     fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
//         Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
//     }
// }

// TODO: ExtractDependencyGraph has been removed
// This was dependent on cim_domain_graph which is no longer available
//
// /// Extract file dependency graph
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExtractDependencyGraph {
//     /// Repository ID
//     pub repository_id: RepositoryId,
//
//     /// Commit to analyze (defaults to HEAD)
//     pub commit_hash: Option<CommitHash>,
//
//     /// File patterns to include
//     pub include_patterns: Vec<String>,
//
//     /// File patterns to exclude
//     pub exclude_patterns: Vec<String>,
//
//     /// Programming language to analyze
//     pub language: Option<String>,
// }
//
// impl Command for ExtractDependencyGraph {
//     type Aggregate = Repository;
//
//     fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
//         Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
//     }
// }

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

impl Command for CreateBranch {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for DeleteBranch {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for CreateTag {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for AnalyzeRepository {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for FetchRemote {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for AnalyzeFileHistory {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for CompareBranches {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for SearchRepository {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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

impl Command for GitHubIntegration {
    type Aggregate = Repository;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.repository_id.as_uuid()))
    }
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
