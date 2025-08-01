// Copyright 2025 Cowboy AI, LLC.

//! Event handling tests for cim-domain-git
//!
//! Tests event creation, serialization, and aggregate event application.
//!
//! ```mermaid
//! graph TD
//!     A[Command] --> B[Aggregate]
//!     B --> C[Domain Events]
//!     C --> D[Event Store]
//!     C --> E[Aggregate State Update]
//! ```

use chrono::Utc;
use cim_domain_git::{
    aggregate::{Repository, RepositoryId},
    events::*,
    value_objects::{AuthorInfo, BranchName, CommitHash, FilePath, RemoteUrl},
};

#[test]
fn test_repository_analyzed_event() {
    let event = RepositoryAnalyzed {
        repository_id: RepositoryId::new(),
        path: "/home/user/repo".to_string(),
        name: "test-repo".to_string(),
        branch_count: 5,
        commit_count: 100,
        timestamp: Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RepositoryAnalyzed = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.repository_id, event.repository_id);
    assert_eq!(deserialized.path, event.path);
    assert_eq!(deserialized.name, event.name);
    assert_eq!(deserialized.branch_count, event.branch_count);
    assert_eq!(deserialized.commit_count, event.commit_count);
}

#[test]
fn test_repository_cloned_event() {
    let event = RepositoryCloned {
        repository_id: RepositoryId::new(),
        remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
        local_path: "/tmp/repo".to_string(),
        timestamp: Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RepositoryCloned = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.repository_id, event.repository_id);
    assert_eq!(deserialized.remote_url, event.remote_url);
    assert_eq!(deserialized.local_path, event.local_path);
}

#[test]
fn test_branch_created_event() {
    let event = BranchCreated {
        repository_id: RepositoryId::new(),
        branch_name: BranchName::new("feature/test").unwrap(),
        commit_hash: CommitHash::new("abc123def456").unwrap(),
        source_branch: Some(BranchName::new("main").unwrap()),
        timestamp: Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: BranchCreated = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.repository_id, event.repository_id);
    assert_eq!(deserialized.branch_name, event.branch_name);
    assert_eq!(deserialized.commit_hash, event.commit_hash);
    assert_eq!(deserialized.source_branch, event.source_branch);
}

#[test]
fn test_commit_analyzed_event() {
    let event = CommitAnalyzed {
        repository_id: RepositoryId::new(),
        commit_hash: CommitHash::new("abc123def456").unwrap(),
        parents: vec![CommitHash::new("def456abc789").unwrap()],
        author: AuthorInfo::new("John Doe".to_string(), "john@example.com".to_string()),
        message: "Test commit".to_string(),
        files_changed: vec![FileChangeInfo {
            path: FilePath::new("src/main.rs").unwrap(),
            change_type: FileChangeType::Modified,
            additions: 10,
            deletions: 5,
        }],
        commit_timestamp: Utc::now(),
        timestamp: Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CommitAnalyzed = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.repository_id, event.repository_id);
    assert_eq!(deserialized.commit_hash, event.commit_hash);
    assert_eq!(deserialized.parents, event.parents);
    assert_eq!(deserialized.author, event.author);
    assert_eq!(deserialized.message, event.message);
    // Note: FileChangeInfo doesn't implement PartialEq, so we check fields individually
    assert_eq!(deserialized.files_changed.len(), event.files_changed.len());
    assert_eq!(
        deserialized.files_changed[0].path,
        event.files_changed[0].path
    );
    assert_eq!(
        deserialized.files_changed[0].change_type,
        event.files_changed[0].change_type
    );
    assert_eq!(
        deserialized.files_changed[0].additions,
        event.files_changed[0].additions
    );
    assert_eq!(
        deserialized.files_changed[0].deletions,
        event.files_changed[0].deletions
    );
}

#[test]
fn test_aggregate_event_application() {
    let mut repo = Repository::new("test-repo".to_string());
    let repo_id = RepositoryId::new();
    repo.id = repo_id;

    // Apply RepositoryCloned event (which the aggregate actually handles)
    let cloned_event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
        repository_id: repo_id,
        remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
        local_path: "/home/user/repo".to_string(),
        timestamp: Utc::now(),
    });

    repo.apply_event(&cloned_event).unwrap();
    assert_eq!(repo.local_path, Some("/home/user/repo".to_string()));
    assert!(repo.remote_url.is_some());

    // Apply BranchCreated event
    let branch_event = GitDomainEvent::BranchCreated(BranchCreated {
        repository_id: repo_id,
        branch_name: BranchName::new("feature/new").unwrap(),
        commit_hash: CommitHash::new("abc123def456789").unwrap(),
        source_branch: None,
        timestamp: Utc::now(),
    });

    repo.apply_event(&branch_event).unwrap();
    assert!(repo
        .branches
        .contains_key(&BranchName::new("feature/new").unwrap()));

    // Apply CommitAnalyzed event
    let commit_event = GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
        repository_id: repo_id,
        commit_hash: CommitHash::new("abc123def456789").unwrap(),
        parents: vec![],
        author: AuthorInfo::new("Test Author".to_string(), "test@example.com".to_string()),
        message: "Test commit".to_string(),
        files_changed: vec![],
        commit_timestamp: Utc::now(),
        timestamp: Utc::now(),
    });

    repo.apply_event(&commit_event).unwrap();
    // Note: Repository doesn't store individual commits, only updates metadata
    assert_eq!(repo.metadata.commit_count, Some(1));
}

// Graph-related event tests have been removed as the graph functionality was removed from the codebase

#[test]
fn test_tag_created_event() {
    let event = TagCreated {
        repository_id: RepositoryId::new(),
        tag_name: cim_domain_git::value_objects::TagName::new("v1.0.0").unwrap(),
        commit_hash: CommitHash::new("abc123def456789").unwrap(),
        message: Some("Release version 1.0.0".to_string()),
        tagger: Some(AuthorInfo::new(
            "Jane Doe".to_string(),
            "jane@example.com".to_string(),
        )),
        timestamp: Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: TagCreated = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.repository_id, event.repository_id);
    assert_eq!(deserialized.tag_name, event.tag_name);
    assert_eq!(deserialized.commit_hash, event.commit_hash);
    assert_eq!(deserialized.message, event.message);
    assert_eq!(deserialized.tagger, event.tagger);
}

#[test]
fn test_event_enum_serialization() {
    let event = GitDomainEvent::RepositoryAnalyzed(RepositoryAnalyzed {
        repository_id: RepositoryId::new(),
        path: "/test".to_string(),
        name: "test".to_string(),
        branch_count: 1,
        commit_count: 1,
        timestamp: Utc::now(),
    });

    // Test that the enum serializes correctly
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: GitDomainEvent = serde_json::from_str(&json).unwrap();

    match deserialized {
        GitDomainEvent::RepositoryAnalyzed(e) => {
            assert_eq!(e.name, "test");
        }
        _ => panic!("Wrong event type deserialized"),
    }
}
