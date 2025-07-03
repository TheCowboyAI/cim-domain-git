//! Integration tests for Git domain with real repositories

use cim_domain_git::{
    commands::*,
    events::{FileChangeType, GitDomainEvent},
    handlers::{extract_dependency_graph, RepositoryCommandHandler},
};
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a test Git repository with some commits
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Create initial commit
    let sig = Signature::now("Test Author", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();

        // Add a file
        let file_path = temp_dir.path().join("README.md");
        fs::write(&file_path, "# Test Repository\n\nThis is a test.").unwrap();
        index.add_path(Path::new("README.md")).unwrap();

        // Add a Rust file
        let rust_file = temp_dir.path().join("main.rs");
        fs::write(
            &rust_file,
            r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

fn main() {
    println!("Hello, world!");
}
"#,
        )
        .unwrap();
        index.add_path(Path::new("main.rs")).unwrap();

        // Add Cargo.toml
        let cargo_file = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_file,
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )
        .unwrap();
        index.add_path(Path::new("Cargo.toml")).unwrap();

        index.write().unwrap();
        index.write_tree().unwrap()
    };

    let tree = repo.find_tree(tree_id).unwrap();
    let initial_commit = repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create a branch
    let head_commit = repo.find_commit(initial_commit).unwrap();
    repo.branch("feature-branch", &head_commit, false).unwrap();

    // Add another commit
    let tree_id2 = {
        let mut index = repo.index().unwrap();
        let file_path = temp_dir.path().join("lib.rs");
        fs::write(
            &file_path,
            r#"
pub mod utils;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .unwrap();
        index.add_path(Path::new("lib.rs")).unwrap();
        index.write().unwrap();
        index.write_tree().unwrap()
    };

    let tree2 = repo.find_tree(tree_id2).unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add library file",
        &tree2,
        &[&head_commit],
    )
    .unwrap();

    temp_dir
}

#[tokio::test]
async fn test_analyze_real_repository() {
    let temp_dir = create_test_repo();
    let handler = RepositoryCommandHandler::new();

    let result = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await;

    assert!(result.is_ok());
    let (repo_id, events) = result.unwrap();

    // Should have repository analyzed event
    assert!(events
        .iter()
        .any(|e| matches!(e, GitDomainEvent::RepositoryAnalyzed(_))));

    // Should have branch events
    let branch_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GitDomainEvent::BranchCreated(_)))
        .collect();
    assert!(!branch_events.is_empty());

    // Should have commit events
    let commit_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GitDomainEvent::CommitAnalyzed(_)))
        .collect();
    assert_eq!(commit_events.len(), 2); // Two commits

    // Verify repository is stored
    let stored_repo = handler.get_repository(&repo_id);
    assert!(stored_repo.is_some());
}

#[tokio::test]
async fn test_extract_commit_graph_from_real_repo() {
    let temp_dir = create_test_repo();
    let handler = RepositoryCommandHandler::new();

    // First analyze the repository
    let (repo_id, _) = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Extract commit graph
    let cmd = ExtractCommitGraph {
        repository_id: repo_id,
        start_commit: None,
        max_depth: Some(10),
        include_all_branches: true,
        include_tags: true,
    };

    let events = handler.extract_commit_graph(cmd).await.unwrap();

    assert_eq!(events.len(), 1);
    if let GitDomainEvent::CommitGraphExtracted(event) = &events[0] {
        assert_eq!(event.repository_id, repo_id);
        assert_eq!(event.commit_count, 2);
        assert_eq!(event.edge_count, 1); // One parent-child relationship
        assert_eq!(event.root_commits.len(), 1); // One root commit
        assert_eq!(event.head_commits.len(), 1); // One head commit
    } else {
        panic!("Expected CommitGraphExtracted event");
    }
}

#[tokio::test]
async fn test_extract_dependency_graph_from_real_repo() {
    let temp_dir = create_test_repo();
    let handler = RepositoryCommandHandler::new();

    // First analyze the repository
    let (repo_id, _) = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Get the repository to access git2 repo
    let repo = handler.get_repository(&repo_id).unwrap();
    let git_repo = Repository::open(repo.local_path.as_ref().unwrap()).unwrap();

    // Extract dependency graph
    let cmd = ExtractDependencyGraph {
        repository_id: repo_id,
        commit_hash: None,
        language: Some("rust".to_string()),
        include_patterns: vec![],
        exclude_patterns: vec![],
    };

    let events = extract_dependency_graph(cmd, &git_repo).await.unwrap();

    assert_eq!(events.len(), 1);
    if let GitDomainEvent::DependencyGraphExtracted(event) = &events[0] {
        assert_eq!(event.repository_id, repo_id);
        assert_eq!(event.file_count, 2); // main.rs and lib.rs
        assert!(event.dependency_count > 0); // Should find serde imports
        assert_eq!(event.language, Some("rust".to_string()));
    } else {
        panic!("Expected DependencyGraphExtracted event");
    }
}

#[tokio::test]
async fn test_file_change_tracking() {
    let temp_dir = create_test_repo();

    // Modify a file and create a new commit
    let repo = Repository::open(temp_dir.path()).unwrap();
    let sig = Signature::now("Test Author", "test@example.com").unwrap();
    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();

    let tree_id = {
        let mut index = repo.index().unwrap();
        let file_path = temp_dir.path().join("README.md");
        fs::write(
            &file_path,
            "# Test Repository\n\nThis is a test.\n\nUpdated!",
        )
        .unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        index.write_tree().unwrap()
    };

    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Update README",
        &tree,
        &[&parent_commit],
    )
    .unwrap();

    // Analyze repository
    let handler = RepositoryCommandHandler::new();
    let (_repo_id, events) = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Find the commit that modified README.md
    let commit_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            GitDomainEvent::CommitAnalyzed(commit) => Some(commit),
            _ => None,
        })
        .collect();

    let update_commit = commit_events
        .iter()
        .find(|c| c.message.contains("Update README"))
        .expect("Should find update commit");

    assert_eq!(update_commit.files_changed.len(), 1);
    assert_eq!(update_commit.files_changed[0].path.as_str(), "README.md");
    assert_eq!(
        update_commit.files_changed[0].change_type,
        FileChangeType::Modified
    );
}

#[tokio::test]
async fn test_branch_operations() {
    let temp_dir = create_test_repo();
    let handler = RepositoryCommandHandler::new();

    // Analyze repository
    let (_repo_id, initial_events) = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Count initial branches
    let initial_branch_count = initial_events
        .iter()
        .filter(|e| matches!(e, GitDomainEvent::BranchCreated(_)))
        .count();

    // Create a new branch using git2 - open a new instance to avoid borrow issues
    {
        let repo = Repository::open(temp_dir.path()).unwrap();
        let head = repo.head().unwrap();
        let head_commit = head.peel_to_commit().unwrap();
        repo.branch("test-branch", &head_commit, false).unwrap();
    }

    // Re-analyze repository
    let (_repo_id2, new_events) = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    let new_branch_count = new_events
        .iter()
        .filter(|e| matches!(e, GitDomainEvent::BranchCreated(_)))
        .count();

    assert_eq!(new_branch_count, initial_branch_count + 1);
}

#[tokio::test]
async fn test_empty_repository() {
    let temp_dir = TempDir::new().unwrap();
    let _repo = Repository::init(temp_dir.path()).unwrap();

    let handler = RepositoryCommandHandler::new();
    let result = handler
        .analyze_repository_at_path(temp_dir.path().to_str().unwrap())
        .await;

    assert!(result.is_ok());
    let (_repo_id, events) = result.unwrap();

    // Should have repository analyzed event even for empty repo
    assert!(events
        .iter()
        .any(|e| matches!(e, GitDomainEvent::RepositoryAnalyzed(_))));

    // Should have no commits
    let commit_count = events
        .iter()
        .filter(|e| matches!(e, GitDomainEvent::CommitAnalyzed(_)))
        .count();
    assert_eq!(commit_count, 0);
}
