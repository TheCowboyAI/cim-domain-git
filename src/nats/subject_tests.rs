// Copyright 2025 Cowboy AI, LLC.

//! Tests for NATS subject mapping

#[cfg(test)]
mod tests {
    use crate::nats::subject::*;

    #[test]
    fn test_command_subjects() {
        assert_eq!(
            GitSubject::command(CommandAction::CloneRepository).to_string(),
            "git.cmd.repository.clone"
        );

        assert_eq!(
            GitSubject::command(CommandAction::AnalyzeCommit).to_string(),
            "git.cmd.commit.analyze"
        );

        assert_eq!(
            GitSubject::command(CommandAction::CreateBranch).to_string(),
            "git.cmd.branch.create"
        );
    }

    #[test]
    fn test_event_subjects() {
        assert_eq!(
            GitSubject::event(EventAction::RepositoryCloned).to_string(),
            "git.event.repository.cloned"
        );

        assert_eq!(
            GitSubject::event(EventAction::CommitAnalyzed).to_string(),
            "git.event.commit.analyzed"
        );

        assert_eq!(
            GitSubject::event(EventAction::BranchCreated).to_string(),
            "git.event.branch.created"
        );
    }

    #[test]
    fn test_query_subjects() {
        assert_eq!(
            GitSubject::query(QueryAction::GetRepository).to_string(),
            "git.query.repository.get"
        );

        assert_eq!(
            GitSubject::query(QueryAction::GetCommitHistory).to_string(),
            "git.query.commit.history"
        );

        assert_eq!(
            GitSubject::query(QueryAction::ListBranches).to_string(),
            "git.query.branch.list"
        );
    }

    #[test]
    fn test_wildcard_subjects() {
        assert_eq!(GitSubject::wildcard(MessageType::Command), "git.cmd.>");

        assert_eq!(GitSubject::wildcard(MessageType::Event), "git.event.>");

        assert_eq!(GitSubject::wildcard(MessageType::Query), "git.query.>");
    }

    #[test]
    fn test_aggregate_wildcard_subjects() {
        assert_eq!(
            GitSubject::aggregate_wildcard(MessageType::Command, Aggregate::Repository),
            "git.cmd.repository.>"
        );

        assert_eq!(
            GitSubject::aggregate_wildcard(MessageType::Event, Aggregate::Commit),
            "git.event.commit.>"
        );

        assert_eq!(
            GitSubject::aggregate_wildcard(MessageType::Query, Aggregate::Branch),
            "git.query.branch.>"
        );
    }

    #[test]
    fn test_subject_mapper_commands() {
        assert!(SubjectMapper::command_subject("CloneRepository").is_some());
        assert!(SubjectMapper::command_subject("AnalyzeCommit").is_some());
        assert!(SubjectMapper::command_subject("CreateBranch").is_some());
        assert!(SubjectMapper::command_subject("UnknownCommand").is_none());
    }

    #[test]
    fn test_subject_mapper_events() {
        assert!(SubjectMapper::event_subject("RepositoryCloned").is_some());
        assert!(SubjectMapper::event_subject("CommitAnalyzed").is_some());
        assert!(SubjectMapper::event_subject("BranchCreated").is_some());
        assert!(SubjectMapper::event_subject("UnknownEvent").is_none());
    }

    #[test]
    fn test_subject_mapper_queries() {
        assert!(SubjectMapper::query_subject("GetRepository").is_some());
        assert!(SubjectMapper::query_subject("GetCommitHistory").is_some());
        assert!(SubjectMapper::query_subject("ListBranches").is_some());
        assert!(SubjectMapper::query_subject("UnknownQuery").is_none());
    }

    #[test]
    fn test_aggregate_display() {
        assert_eq!(Aggregate::Repository.to_string(), "repository");
        assert_eq!(Aggregate::Commit.to_string(), "commit");
        assert_eq!(Aggregate::Branch.to_string(), "branch");
        assert_eq!(Aggregate::Tag.to_string(), "tag");
        assert_eq!(Aggregate::Remote.to_string(), "remote");
    }

    #[test]
    fn test_message_type_display() {
        assert_eq!(MessageType::Command.to_string(), "cmd");
        assert_eq!(MessageType::Event.to_string(), "event");
        assert_eq!(MessageType::Query.to_string(), "query");
    }
}
