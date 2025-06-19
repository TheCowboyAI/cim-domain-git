//! Domain events for the Git domain
//!
//! Events represent facts that have occurred in the Git domain.
//! All events are immutable and represent past occurrences.

use crate::aggregate::RepositoryId;
use crate::value_objects::{AuthorInfo, BranchName, CommitHash, FilePath, RemoteUrl, TagName};
use chrono::{DateTime, Utc};
use cim_domain_graph::GraphId;
use serde::{Deserialize, Serialize};

/// Enumeration of all Git domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum GitDomainEvent {
    /// A repository was cloned
    RepositoryCloned(RepositoryCloned),

    /// A commit was analyzed
    CommitAnalyzed(CommitAnalyzed),

    /// A branch was created
    BranchCreated(BranchCreated),

    /// A branch was deleted
    BranchDeleted(BranchDeleted),

    /// A tag was created
    TagCreated(TagCreated),

    /// A commit graph was extracted
    CommitGraphExtracted(CommitGraphExtracted),

    /// A file dependency graph was extracted
    DependencyGraphExtracted(DependencyGraphExtracted),

    /// Repository metadata was updated
    RepositoryMetadataUpdated(RepositoryMetadataUpdated),

    /// A merge was detected
    MergeDetected(MergeDetected),

    /// A file was analyzed
    FileAnalyzed(FileAnalyzed),

    /// A repository was analyzed
    RepositoryAnalyzed(RepositoryAnalyzed),
}

/// Event: A repository was cloned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryCloned {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Remote URL
    pub remote_url: RemoteUrl,

    /// Local path where cloned
    pub local_path: String,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A commit was analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAnalyzed {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Commit hash
    pub commit_hash: CommitHash,

    /// Parent commits
    pub parents: Vec<CommitHash>,

    /// Author information
    pub author: AuthorInfo,

    /// Commit message
    pub message: String,

    /// Files changed
    pub files_changed: Vec<FileChangeInfo>,

    /// Timestamp of the commit
    pub commit_timestamp: DateTime<Utc>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Information about a file change in a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeInfo {
    /// File path
    pub path: FilePath,

    /// Type of change
    pub change_type: FileChangeType,

    /// Lines added
    pub additions: usize,

    /// Lines deleted
    pub deletions: usize,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

/// Event: A branch was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCreated {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Branch name
    pub branch_name: BranchName,

    /// Commit the branch points to
    pub commit_hash: CommitHash,

    /// Source branch (if branched from another)
    pub source_branch: Option<BranchName>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A branch was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDeleted {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Branch name
    pub branch_name: BranchName,

    /// Last commit the branch pointed to
    pub last_commit: CommitHash,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A tag was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCreated {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Tag name
    pub tag_name: TagName,

    /// Commit the tag points to
    pub commit_hash: CommitHash,

    /// Tag message (if annotated)
    pub message: Option<String>,

    /// Tagger information (if annotated)
    pub tagger: Option<AuthorInfo>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A commit graph was extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraphExtracted {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Graph ID in the graph domain
    pub graph_id: GraphId,

    /// Number of commits (nodes) in the graph
    pub commit_count: usize,

    /// Number of parent-child relationships (edges)
    pub edge_count: usize,

    /// Root commits (no parents)
    pub root_commits: Vec<CommitHash>,

    /// Head commits (no children)
    pub head_commits: Vec<CommitHash>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A file dependency graph was extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphExtracted {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Graph ID in the graph domain
    pub graph_id: GraphId,

    /// Commit hash this analysis is based on
    pub commit_hash: CommitHash,

    /// Number of files (nodes) in the graph
    pub file_count: usize,

    /// Number of dependencies (edges)
    pub dependency_count: usize,

    /// Programming language analyzed
    pub language: Option<String>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: Repository metadata was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadataUpdated {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Updated fields
    pub updates: MetadataUpdates,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Metadata updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataUpdates {
    /// Updated description
    pub description: Option<String>,

    /// Updated primary language
    pub primary_language: Option<String>,

    /// Updated size in bytes
    pub size_bytes: Option<u64>,

    /// Updated commit count
    pub commit_count: Option<usize>,

    /// Custom metadata updates
    pub custom: Option<serde_json::Value>,
}

/// Event: A merge was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeDetected {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Merge commit hash
    pub merge_commit: CommitHash,

    /// Parent commits that were merged
    pub parents: Vec<CommitHash>,

    /// Branches involved (if known)
    pub branches: Vec<BranchName>,

    /// Merge strategy used (if known)
    pub merge_strategy: Option<String>,

    /// Conflicts that occurred
    pub conflicts: Vec<FilePath>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Event: A file was analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalyzed {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// File path
    pub file_path: FilePath,

    /// Commit hash where analyzed
    pub commit_hash: CommitHash,

    /// File metrics
    pub metrics: FileMetrics,

    /// Dependencies found
    pub dependencies: Vec<FilePath>,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Metrics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    /// Lines of code
    pub lines_of_code: usize,

    /// Number of functions/methods
    pub function_count: Option<usize>,

    /// Cyclomatic complexity
    pub complexity: Option<u32>,

    /// Programming language
    pub language: Option<String>,

    /// File size in bytes
    pub size_bytes: u64,
}

/// Event: A repository was analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalyzed {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Repository path
    pub path: String,

    /// Repository name
    pub name: String,

    /// Number of branches found
    pub branch_count: usize,

    /// Number of commits analyzed
    pub commit_count: usize,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: GitDomainEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            GitDomainEvent::RepositoryCloned(e) => {
                assert_eq!(e.local_path, "/tmp/repo");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
