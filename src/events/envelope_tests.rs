// Copyright 2025 Cowboy AI, LLC.

//! Tests for event envelope functionality

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::aggregate::RepositoryId;
    use crate::value_objects::RemoteUrl;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_event_envelope_new() {
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event.clone());
        
        assert_eq!(envelope.event_type(), "RepositoryCloned");
        assert_eq!(envelope.correlation_id(), envelope.event_id());
        assert_eq!(envelope.causation_id(), envelope.event_id());
    }

    #[test]
    fn test_event_envelope_from_command() {
        let command_id = Uuid::new_v4();
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::from_command(event, command_id);
        
        assert_eq!(envelope.causation_id(), command_id);
        assert_eq!(envelope.correlation_id(), command_id);
    }

    #[test]
    fn test_event_envelope_from_correlation() {
        let correlation_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::from_correlation(event, correlation_id, causation_id);
        
        assert_eq!(envelope.correlation_id(), correlation_id);
        assert_eq!(envelope.causation_id(), causation_id);
    }

    #[test]
    fn test_event_envelope_with_metadata() {
        let metadata = EventMetadata::new().with_user("test-user".to_string());
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::with_metadata(event, metadata.clone());
        
        assert_eq!(envelope.metadata.user_id, Some("test-user".to_string()));
        assert_eq!(envelope.metadata.event_id, metadata.event_id);
    }

    #[test]
    fn test_all_event_types() {
        let repo_id = RepositoryId::new();
        
        // Test each event type
        let events = vec![
            GitDomainEvent::RepositoryCloned(RepositoryCloned {
                repository_id: repo_id,
                remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
                local_path: "/tmp/repo".to_string(),
                timestamp: Utc::now(),
            }),
            GitDomainEvent::CommitAnalyzed(CommitAnalyzed {
                repository_id: repo_id,
                commit_hash: crate::value_objects::CommitHash::new("abc123d").unwrap(),
                parents: vec![],
                author: crate::value_objects::AuthorInfo {
                    name: "Test Author".to_string(),
                    email: "test@example.com".to_string(),
                },
                message: "Test commit".to_string(),
                files_changed: vec![],
                commit_timestamp: Utc::now(),
                timestamp: Utc::now(),
            }),
            GitDomainEvent::BranchCreated(BranchCreated {
                repository_id: repo_id,
                branch_name: crate::value_objects::BranchName::new("feature/test").unwrap(),
                commit_hash: crate::value_objects::CommitHash::new("abc123d").unwrap(),
                source_branch: None,
                timestamp: Utc::now(),
            }),
            GitDomainEvent::BranchDeleted(BranchDeleted {
                repository_id: repo_id,
                branch_name: crate::value_objects::BranchName::new("feature/old").unwrap(),
                last_commit: crate::value_objects::CommitHash::new("def456a").unwrap(),
                timestamp: Utc::now(),
            }),
            GitDomainEvent::TagCreated(TagCreated {
                repository_id: repo_id,
                tag_name: crate::value_objects::TagName::new("v1.0.0").unwrap(),
                commit_hash: crate::value_objects::CommitHash::new("abc123d").unwrap(),
                message: Some("Release v1.0.0".to_string()),
                tagger: None,
                timestamp: Utc::now(),
            }),
            GitDomainEvent::RepositoryMetadataUpdated(RepositoryMetadataUpdated {
                repository_id: repo_id,
                updates: MetadataUpdates {
                    description: Some("Updated description".to_string()),
                    primary_language: Some("Rust".to_string()),
                    size_bytes: Some(1024),
                    commit_count: Some(100),
                    custom: None,
                },
                timestamp: Utc::now(),
            }),
            GitDomainEvent::MergeDetected(MergeDetected {
                repository_id: repo_id,
                merge_commit: crate::value_objects::CommitHash::new("123abc4").unwrap(),
                parents: vec![
                    crate::value_objects::CommitHash::new("1234567").unwrap(),
                    crate::value_objects::CommitHash::new("abcdef0").unwrap(),
                ],
                branches: vec![],
                merge_strategy: Some("recursive".to_string()),
                conflicts: vec![],
                timestamp: Utc::now(),
            }),
            GitDomainEvent::FileAnalyzed(FileAnalyzed {
                repository_id: repo_id,
                file_path: crate::value_objects::FilePath::new("src/main.rs").unwrap(),
                commit_hash: crate::value_objects::CommitHash::new("abc123d").unwrap(),
                metrics: FileMetrics {
                    lines_of_code: 100,
                    function_count: Some(5),
                    complexity: Some(10),
                    language: Some("Rust".to_string()),
                    size_bytes: 2048,
                },
                dependencies: vec![],
                timestamp: Utc::now(),
            }),
            GitDomainEvent::RepositoryAnalyzed(RepositoryAnalyzed {
                repository_id: repo_id,
                path: "/tmp/repo".to_string(),
                name: "test-repo".to_string(),
                branch_count: 5,
                commit_count: 100,
                timestamp: Utc::now(),
            }),
        ];

        // Test event type detection and aggregate ID extraction
        let expected_types = vec![
            "RepositoryCloned",
            "CommitAnalyzed",
            "BranchCreated",
            "BranchDeleted",
            "TagCreated",
            "RepositoryMetadataUpdated",
            "MergeDetected",
            "FileAnalyzed",
            "RepositoryAnalyzed",
        ];

        for (event, expected_type) in events.iter().zip(expected_types.iter()) {
            let envelope = EventEnvelope::new(event.clone());
            assert_eq!(envelope.event_type(), *expected_type);
            assert_eq!(envelope.aggregate_id(), repo_id.to_string());
        }
    }

    #[test]
    fn test_event_envelope_serialization() {
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/repo".to_string(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::new(event);
        
        // Serialize and deserialize
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.event_id(), envelope.event_id());
        assert_eq!(deserialized.event_type(), envelope.event_type());
        assert_eq!(deserialized.aggregate_id(), envelope.aggregate_id());
    }

    #[test]
    fn test_file_change_info() {
        let change = FileChangeInfo {
            path: crate::value_objects::FilePath::new("src/lib.rs").unwrap(),
            change_type: FileChangeType::Modified,
            additions: 10,
            deletions: 5,
        };

        assert_eq!(change.additions, 10);
        assert_eq!(change.deletions, 5);
        assert_eq!(change.change_type, FileChangeType::Modified);
    }

    #[test]
    fn test_file_change_types() {
        let types = vec![
            FileChangeType::Added,
            FileChangeType::Modified,
            FileChangeType::Deleted,
            FileChangeType::Renamed,
        ];

        for change_type in types {
            let change = FileChangeInfo {
                path: crate::value_objects::FilePath::new("test.rs").unwrap(),
                change_type,
                additions: 0,
                deletions: 0,
            };

            match change.change_type {
                FileChangeType::Added => assert_eq!(change_type, FileChangeType::Added),
                FileChangeType::Modified => assert_eq!(change_type, FileChangeType::Modified),
                FileChangeType::Deleted => assert_eq!(change_type, FileChangeType::Deleted),
                FileChangeType::Renamed => assert_eq!(change_type, FileChangeType::Renamed),
            }
        }
    }
}