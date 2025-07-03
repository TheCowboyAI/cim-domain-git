//! Basic tests for cim-domain-git module structure

use cim_domain_git::{
    aggregate::Repository,
    commands::CloneRepository,
    value_objects::{BranchName, CommitHash, RemoteUrl},
    GitDomainError,
};

#[test]
fn test_repository_creation() {
    let repo = Repository::new("test-repo".to_string());
    assert_eq!(repo.metadata.name, "test-repo");
    assert!(repo.remote_url.is_none());
    assert!(repo.local_path.is_none());
}

#[test]
fn test_value_objects() {
    // Test RemoteUrl
    let url = RemoteUrl::new("https://github.com/test/repo.git").unwrap();
    assert_eq!(url.repository_name(), Some("repo"));
    assert!(url.is_github());

    // Test CommitHash
    let hash = CommitHash::new("abc123def456").unwrap();
    assert_eq!(hash.short(), "abc123d");

    // Test BranchName
    let branch = BranchName::new("main").unwrap();
    assert!(branch.is_default());
}

#[test]
fn test_command_creation() {
    let url = RemoteUrl::new("https://github.com/test/repo.git").unwrap();
    let cmd = CloneRepository {
        repository_id: None,
        remote_url: url,
        local_path: "/tmp/repo".to_string(),
        branch: None,
        depth: None,
    };

    assert_eq!(cmd.local_path, "/tmp/repo");
}

#[test]
fn test_error_types() {
    let err = GitDomainError::RepositoryNotFound("test".to_string());
    assert_eq!(err.to_string(), "Repository not found: test");

    let err = GitDomainError::InvalidCommitHash("xyz".to_string());
    assert_eq!(err.to_string(), "Invalid commit hash: xyz");
}
