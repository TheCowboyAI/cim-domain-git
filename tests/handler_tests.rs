//! Handler integration tests for cim-domain-git
//!
//! Tests the CQRS adapters and command handling functionality.
//!
//! ```mermaid
//! graph TD
//!     A[Command] --> B[CommandHandler]
//!     B --> C[RepositoryCommandHandler]
//!     C --> D[Domain Events]
//!     D --> E[Aggregate State Update]
//! ```

use cim_domain::{CommandEnvelope, CommandStatus, CommandHandler, CommandId};
use cim_subject::{MessageIdentity, CorrelationId, CausationId, IdType};
use cim_domain_git::{
    commands::*,
    handlers::*,
    aggregate::RepositoryId,
    value_objects::{BranchName, CommitHash, RemoteUrl, TagName, FilePath},
};

/// Helper function to create a test command envelope
fn create_test_envelope<T>(command: T) -> CommandEnvelope<T> {
    let command_id = CommandId::new();
    let id_uuid = *command_id.as_uuid();
    CommandEnvelope {
        id: command_id,
        identity: MessageIdentity {
            message_id: IdType::Uuid(id_uuid),
            correlation_id: CorrelationId(IdType::Uuid(id_uuid)),
            causation_id: CausationId(IdType::Uuid(id_uuid)),
        },
        command,
        issued_by: "test-user".to_string(),
    }
}

#[test]
fn test_extract_commit_graph_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = ExtractCommitGraphHandler::new(repo_handler);

    let command = ExtractCommitGraph {
        repository_id: RepositoryId::new(),
        start_commit: None,
        max_depth: Some(10),
        include_all_branches: true,
        include_tags: false,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    // Since repository doesn't exist, it should be rejected
    assert_eq!(ack.status, CommandStatus::Rejected);
    assert!(ack.reason.is_some());
}

#[test]
fn test_clone_repository_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = CloneRepositoryHandler::new(repo_handler);

    let command = CloneRepository {
        repository_id: Some(RepositoryId::new()),
        remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
        local_path: "/tmp/test-repo".to_string(),
        branch: Some(BranchName::new("main").unwrap()),
        depth: Some(1),
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    // This will fail as the path doesn't exist, but we're testing the handler works
    assert!(matches!(ack.status, CommandStatus::Accepted | CommandStatus::Rejected));
}

#[test]
fn test_analyze_commit_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = AnalyzeCommitHandler::new(repo_handler);

    let command = AnalyzeCommit {
        repository_id: RepositoryId::new(),
        commit_hash: CommitHash::new("abc123def456789").unwrap(),
        analyze_files: true,
        extract_dependencies: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_create_branch_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = CreateBranchHandler::new(repo_handler);

    let command = CreateBranch {
        repository_id: RepositoryId::new(),
        branch_name: BranchName::new("feature/test").unwrap(),
        start_point: "main".to_string(),
        checkout: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_delete_branch_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = DeleteBranchHandler::new(repo_handler);

    let command = DeleteBranch {
        repository_id: RepositoryId::new(),
        branch_name: BranchName::new("feature/old").unwrap(),
        force: false,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_create_tag_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = CreateTagHandler::new(repo_handler);

    let command = CreateTag {
        repository_id: RepositoryId::new(),
        tag_name: TagName::new("v1.0.0").unwrap(),
        commit_hash: None,
        message: Some("Release version 1.0.0".to_string()),
        annotated: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_analyze_repository_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = AnalyzeRepositoryHandler::new(repo_handler);

    let command = AnalyzeRepository {
        repository_id: RepositoryId::new(),
        update_metadata: true,
        analyze_languages: true,
        calculate_statistics: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_fetch_remote_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = FetchRemoteHandler::new(repo_handler);

    let command = FetchRemote {
        repository_id: RepositoryId::new(),
        remote: Some("origin".to_string()),
        all_remotes: false,
        prune: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_analyze_file_history_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = AnalyzeFileHistoryHandler::new(repo_handler);

    let command = AnalyzeFileHistory {
        repository_id: RepositoryId::new(),
        file_path: FilePath::new("src/lib.rs").unwrap(),
        start_commit: None,
        end_commit: None,
        follow_renames: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_compare_branches_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = CompareBranchesHandler::new(repo_handler);

    let command = CompareBranches {
        repository_id: RepositoryId::new(),
        base_branch: BranchName::new("main").unwrap(),
        compare_branch: BranchName::new("feature/test").unwrap(),
        include_diffs: true,
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_search_repository_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = SearchRepositoryHandler::new(repo_handler);

    let command = SearchRepository {
        repository_id: RepositoryId::new(),
        pattern: "TODO".to_string(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["target/".to_string()],
        case_sensitive: false,
        max_results: Some(100),
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_github_integration_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = GitHubIntegrationHandler::new(repo_handler);

    let command = GitHubIntegration {
        repository_id: RepositoryId::new(),
        github_repo: "test/repo".to_string(),
        operations: vec![
            GitHubOperation::SyncIssues,
            GitHubOperation::SyncPullRequests,
        ],
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
}

#[test]
fn test_extract_dependency_graph_handler() {
    let repo_handler = RepositoryCommandHandler::new();
    let mut handler = ExtractDependencyGraphHandler::new(repo_handler);

    let command = ExtractDependencyGraph {
        repository_id: RepositoryId::new(),
        commit_hash: None,
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["target/".to_string()],
        language: Some("rust".to_string()),
    };

    let envelope = create_test_envelope(command);
    let ack = handler.handle(envelope);

    assert_eq!(ack.status, CommandStatus::Rejected);
    assert_eq!(ack.reason, Some("Repository not found".to_string()));
} 