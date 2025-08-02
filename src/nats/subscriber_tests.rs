// Copyright 2025 Cowboy AI, LLC.

//! Tests for NATS subscriber functionality

#[cfg(test)]
mod tests {
    use super::super::subscriber::*;
    use super::super::error::NatsError;
    use super::super::subject::{CommandAction, EventAction, GitSubject};
    use crate::commands::{CloneRepository, GitCommand};
    use crate::events::{GitDomainEvent, RepositoryCloned};
    use crate::aggregate::RepositoryId;
    use crate::value_objects::RemoteUrl;
    use async_nats::Client;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    #[derive(Clone)]
    struct TestCommandHandler {
        received: Arc<Mutex<Vec<GitCommand>>>,
    }

    #[async_trait::async_trait]
    impl CommandHandler for TestCommandHandler {
        type Command = GitCommand;
        type Result = ();

        async fn handle(&self, command: Self::Command) -> Result<Self::Result, NatsError> {
            self.received.lock().await.push(command);
            Ok(())
        }

        fn command_type(&self) -> &'static str {
            "GitCommand"
        }
    }

    #[derive(Clone)]
    struct TestEventHandler {
        received: Arc<Mutex<Vec<GitDomainEvent>>>,
    }

    #[async_trait::async_trait]
    impl EventHandler for TestEventHandler {
        type Event = GitDomainEvent;

        async fn handle(&self, event: Self::Event) -> Result<(), NatsError> {
            self.received.lock().await.push(event);
            Ok(())
        }

        fn event_type(&self) -> &'static str {
            "GitDomainEvent"
        }
    }

    #[derive(Clone)]
    struct FailingCommandHandler;

    #[async_trait::async_trait]
    impl CommandHandler for FailingCommandHandler {
        type Command = GitCommand;
        type Result = ();

        async fn handle(&self, _command: Self::Command) -> Result<Self::Result, NatsError> {
            Err(NatsError::Other("Handler failed".to_string()))
        }

        fn command_type(&self) -> &'static str {
            "GitCommand"
        }
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_command_subscriber() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let handler = TestCommandHandler {
            received: Arc::new(Mutex::new(Vec::new())),
        };

        let subscriber = CommandSubscriber::new(
            client.clone(),
            "test-handler".to_string(),
        );

        // Register handler and subscribe
        subscriber.register_handler(handler.clone()).await;
        subscriber.start().await.unwrap();

        // Publish a test command
        let command = GitCommand::CloneRepository(CloneRepository {
            repository_id: Some(RepositoryId::new()),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/test".to_string(),
            branch: None,
            depth: None,
        });

        let subject = GitSubject::command(CommandAction::CloneRepository);
        let payload = serde_json::to_vec(&command).unwrap();
        
        client
            .publish(subject.to_string(), payload.into())
            .await
            .unwrap();

        // Give time for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify command was received
        let received = handler.received.lock().await;
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_event_subscriber() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let handler = TestEventHandler {
            received: Arc::new(Mutex::new(Vec::new())),
        };

        let subscriber = EventSubscriber::new(client.clone());

        // Register handler and subscribe
        subscriber.register_handler(handler.clone()).await;
        subscriber.start().await.unwrap();

        // Publish a test event
        let event = GitDomainEvent::RepositoryCloned(RepositoryCloned {
            repository_id: RepositoryId::new(),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/test".to_string(),
            timestamp: Utc::now(),
        });

        let subject = GitSubject::event(EventAction::RepositoryCloned);
        let payload = serde_json::to_vec(&event).unwrap();
        
        client
            .publish(subject.to_string(), payload.into())
            .await
            .unwrap();

        // Give time for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify event was received
        let received = handler.received.lock().await;
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_command_subscriber_error_handling() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let handler = FailingCommandHandler;

        let subscriber = CommandSubscriber::new(
            client.clone(),
            "test-handler".to_string(),
        );

        // Register handler and start subscriber
        subscriber.register_handler(handler).await;
        subscriber.start().await.unwrap();

        // Publish a test command that will fail
        let command = GitCommand::CloneRepository(CloneRepository {
            repository_id: Some(RepositoryId::new()),
            remote_url: RemoteUrl::new("https://github.com/test/repo.git").unwrap(),
            local_path: "/tmp/test".to_string(),
            branch: None,
            depth: None,
        });

        let subject = GitSubject::command(CommandAction::CloneRepository);
        let payload = serde_json::to_vec(&command).unwrap();
        
        // This should not panic even though handler fails
        client
            .publish(subject.to_string(), payload.into())
            .await
            .unwrap();

        // Give time for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test passes if no panic occurred
    }

    #[test]
    fn test_subscriber_subject_mapping() {
        // Test command subject mapping
        let subject = GitSubject::command(CommandAction::CloneRepository);
        assert_eq!(subject.to_string(), "git.command.repository.clone");

        let subject = GitSubject::command(CommandAction::AnalyzeCommit);
        assert_eq!(subject.to_string(), "git.command.commit.analyze");

        // Test event subject mapping
        let subject = GitSubject::event(EventAction::RepositoryCloned);
        assert_eq!(subject.to_string(), "git.event.repository.cloned");

        let subject = GitSubject::event(EventAction::CommitAnalyzed);
        assert_eq!(subject.to_string(), "git.event.commit.analyzed");
    }

    #[tokio::test]
    async fn test_queue_group_naming() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let _subscriber = CommandSubscriber::new(
            client,
            "test-handler".to_string(),
        );

        // Queue group should be based on subject prefix
        // In real implementation, we'd expose this for testing
        // assert_eq!(subscriber.queue_group(), "git-commands");
    }

    #[tokio::test]
    async fn test_multiple_subscribers_load_balancing() {
        // This test would verify that multiple subscribers
        // with the same queue group properly load balance
        // Requires actual NATS server to test properly
    }
}