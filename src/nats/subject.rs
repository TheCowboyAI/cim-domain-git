// Copyright 2025 Cowboy AI, LLC.

//! Subject mapping for the Git domain
//!
//! Defines all NATS subjects used by the Git domain following CIM conventions:
//! - Commands: git.cmd.{aggregate}.{action}
//! - Events: git.event.{aggregate}.{action}
//! - Queries: git.query.{aggregate}.{action}

use std::fmt;

/// The Git domain identifier
pub const DOMAIN: &str = "git";

/// Aggregate types in the Git domain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    Repository,
    Commit,
    Branch,
    Tag,
    Remote,
}

impl fmt::Display for Aggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Aggregate::Repository => write!(f, "repository"),
            Aggregate::Commit => write!(f, "commit"),
            Aggregate::Branch => write!(f, "branch"),
            Aggregate::Tag => write!(f, "tag"),
            Aggregate::Remote => write!(f, "remote"),
        }
    }
}

/// Message types following CIM conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Command,
    Event,
    Query,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Command => write!(f, "cmd"),
            MessageType::Event => write!(f, "event"),
            MessageType::Query => write!(f, "query"),
        }
    }
}

/// Command actions for each aggregate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    // Repository commands
    CloneRepository,
    DeleteRepository,
    
    // Commit commands
    AnalyzeCommit,
    
    // Branch commands
    CreateBranch,
    DeleteBranch,
    MergeBranch,
    
    // Tag commands
    CreateTag,
    DeleteTag,
    
    // Remote commands
    AddRemote,
    RemoveRemote,
    FetchRemote,
    PushRemote,
}

impl CommandAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Repository commands
            CommandAction::CloneRepository => "clone",
            CommandAction::DeleteRepository => "delete",
            
            // Commit commands
            CommandAction::AnalyzeCommit => "analyze",
            
            // Branch commands
            CommandAction::CreateBranch => "create",
            CommandAction::DeleteBranch => "delete",
            CommandAction::MergeBranch => "merge",
            
            // Tag commands
            CommandAction::CreateTag => "create",
            CommandAction::DeleteTag => "delete",
            
            // Remote commands
            CommandAction::AddRemote => "add",
            CommandAction::RemoveRemote => "remove",
            CommandAction::FetchRemote => "fetch",
            CommandAction::PushRemote => "push",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            CommandAction::CloneRepository
            | CommandAction::DeleteRepository => Aggregate::Repository,
            
            CommandAction::AnalyzeCommit => Aggregate::Commit,
            
            CommandAction::CreateBranch
            | CommandAction::DeleteBranch
            | CommandAction::MergeBranch => Aggregate::Branch,
            
            CommandAction::CreateTag
            | CommandAction::DeleteTag => Aggregate::Tag,
            
            CommandAction::AddRemote
            | CommandAction::RemoveRemote
            | CommandAction::FetchRemote
            | CommandAction::PushRemote => Aggregate::Remote,
        }
    }
}

/// Event actions (past tense of commands)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAction {
    // Repository events
    RepositoryCloned,
    RepositoryDeleted,
    RepositoryAnalyzed,
    
    // Commit events
    CommitAnalyzed,
    
    // Branch events
    BranchCreated,
    BranchDeleted,
    BranchMerged,
    
    // Tag events
    TagCreated,
    TagDeleted,
    
    // Remote events
    RemoteAdded,
    RemoteRemoved,
    RemoteFetched,
    RemotePushed,
    
    // File events
    FileAnalyzed,
    
    // Merge events
    MergeDetected,
}

impl EventAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Repository events
            EventAction::RepositoryCloned => "cloned",
            EventAction::RepositoryDeleted => "deleted",
            EventAction::RepositoryAnalyzed => "analyzed",
            
            // Commit events
            EventAction::CommitAnalyzed => "analyzed",
            
            // Branch events
            EventAction::BranchCreated => "created",
            EventAction::BranchDeleted => "deleted",
            EventAction::BranchMerged => "merged",
            
            // Tag events
            EventAction::TagCreated => "created",
            EventAction::TagDeleted => "deleted",
            
            // Remote events
            EventAction::RemoteAdded => "added",
            EventAction::RemoteRemoved => "removed",
            EventAction::RemoteFetched => "fetched",
            EventAction::RemotePushed => "pushed",
            
            // File events
            EventAction::FileAnalyzed => "analyzed",
            
            // Merge events
            EventAction::MergeDetected => "detected",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            EventAction::RepositoryCloned
            | EventAction::RepositoryDeleted
            | EventAction::RepositoryAnalyzed => Aggregate::Repository,
            
            EventAction::CommitAnalyzed
            | EventAction::FileAnalyzed
            | EventAction::MergeDetected => Aggregate::Commit,
            
            EventAction::BranchCreated
            | EventAction::BranchDeleted
            | EventAction::BranchMerged => Aggregate::Branch,
            
            EventAction::TagCreated
            | EventAction::TagDeleted => Aggregate::Tag,
            
            EventAction::RemoteAdded
            | EventAction::RemoteRemoved
            | EventAction::RemoteFetched
            | EventAction::RemotePushed => Aggregate::Remote,
        }
    }
}

/// Query actions for reading data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryAction {
    // Repository queries
    GetRepository,
    ListRepositories,
    GetRepositoryDetails,
    
    // Commit queries
    GetCommit,
    GetCommitHistory,
    
    // Branch queries
    GetBranch,
    ListBranches,
    
    // Tag queries
    GetTag,
    ListTags,
    
    // File queries
    GetFileChanges,
}

impl QueryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Repository queries
            QueryAction::GetRepository => "get",
            QueryAction::ListRepositories => "list",
            QueryAction::GetRepositoryDetails => "details",
            
            // Commit queries
            QueryAction::GetCommit => "get",
            QueryAction::GetCommitHistory => "history",
            
            // Branch queries
            QueryAction::GetBranch => "get",
            QueryAction::ListBranches => "list",
            
            // Tag queries
            QueryAction::GetTag => "get",
            QueryAction::ListTags => "list",
            
            // File queries
            QueryAction::GetFileChanges => "changes",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            QueryAction::GetRepository
            | QueryAction::ListRepositories
            | QueryAction::GetRepositoryDetails => Aggregate::Repository,
            
            QueryAction::GetCommit
            | QueryAction::GetCommitHistory
            | QueryAction::GetFileChanges => Aggregate::Commit,
            
            QueryAction::GetBranch
            | QueryAction::ListBranches => Aggregate::Branch,
            
            QueryAction::GetTag
            | QueryAction::ListTags => Aggregate::Tag,
        }
    }
}

/// NATS subject for Git domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSubject {
    message_type: MessageType,
    aggregate: Aggregate,
    action: String,
}

impl GitSubject {
    /// Create a command subject
    pub fn command(action: CommandAction) -> Self {
        Self {
            message_type: MessageType::Command,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Create an event subject
    pub fn event(action: EventAction) -> Self {
        Self {
            message_type: MessageType::Event,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Create a query subject
    pub fn query(action: QueryAction) -> Self {
        Self {
            message_type: MessageType::Query,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Create a wildcard subject for subscriptions
    pub fn wildcard(message_type: MessageType) -> String {
        format!("{}.{}.>", DOMAIN, message_type)
    }
    
    /// Create an aggregate-specific wildcard
    pub fn aggregate_wildcard(message_type: MessageType, aggregate: Aggregate) -> String {
        format!("{}.{}.{}.>", DOMAIN, message_type, aggregate)
    }
}

impl fmt::Display for GitSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", DOMAIN, self.message_type, self.aggregate, self.action)
    }
}

/// Subject mapper for converting between event types and subjects
pub struct SubjectMapper;

impl SubjectMapper {
    /// Map an event type string to a subject
    pub fn event_subject(event_type: &str) -> Option<GitSubject> {
        match event_type {
            "RepositoryCloned" => Some(GitSubject::event(EventAction::RepositoryCloned)),
            "RepositoryDeleted" => Some(GitSubject::event(EventAction::RepositoryDeleted)),
            "RepositoryAnalyzed" => Some(GitSubject::event(EventAction::RepositoryAnalyzed)),
            "CommitAnalyzed" => Some(GitSubject::event(EventAction::CommitAnalyzed)),
            "BranchCreated" => Some(GitSubject::event(EventAction::BranchCreated)),
            "BranchDeleted" => Some(GitSubject::event(EventAction::BranchDeleted)),
            "BranchMerged" => Some(GitSubject::event(EventAction::BranchMerged)),
            "TagCreated" => Some(GitSubject::event(EventAction::TagCreated)),
            "TagDeleted" => Some(GitSubject::event(EventAction::TagDeleted)),
            "RemoteAdded" => Some(GitSubject::event(EventAction::RemoteAdded)),
            "RemoteRemoved" => Some(GitSubject::event(EventAction::RemoteRemoved)),
            "RemoteFetched" => Some(GitSubject::event(EventAction::RemoteFetched)),
            "RemotePushed" => Some(GitSubject::event(EventAction::RemotePushed)),
            "FileAnalyzed" => Some(GitSubject::event(EventAction::FileAnalyzed)),
            "MergeDetected" => Some(GitSubject::event(EventAction::MergeDetected)),
            _ => None,
        }
    }
    
    /// Map a command type to a subject
    pub fn command_subject(command_type: &str) -> Option<GitSubject> {
        match command_type {
            "CloneRepository" => Some(GitSubject::command(CommandAction::CloneRepository)),
            "DeleteRepository" => Some(GitSubject::command(CommandAction::DeleteRepository)),
            "AnalyzeCommit" => Some(GitSubject::command(CommandAction::AnalyzeCommit)),
            "CreateBranch" => Some(GitSubject::command(CommandAction::CreateBranch)),
            "DeleteBranch" => Some(GitSubject::command(CommandAction::DeleteBranch)),
            "MergeBranch" => Some(GitSubject::command(CommandAction::MergeBranch)),
            "CreateTag" => Some(GitSubject::command(CommandAction::CreateTag)),
            "DeleteTag" => Some(GitSubject::command(CommandAction::DeleteTag)),
            "AddRemote" => Some(GitSubject::command(CommandAction::AddRemote)),
            "RemoveRemote" => Some(GitSubject::command(CommandAction::RemoveRemote)),
            "FetchRemote" => Some(GitSubject::command(CommandAction::FetchRemote)),
            "PushRemote" => Some(GitSubject::command(CommandAction::PushRemote)),
            _ => None,
        }
    }
    
    /// Map a query type to a subject
    pub fn query_subject(query_type: &str) -> Option<GitSubject> {
        match query_type {
            "GetRepository" => Some(GitSubject::query(QueryAction::GetRepository)),
            "ListRepositories" => Some(GitSubject::query(QueryAction::ListRepositories)),
            "GetRepositoryDetails" => Some(GitSubject::query(QueryAction::GetRepositoryDetails)),
            "GetCommit" => Some(GitSubject::query(QueryAction::GetCommit)),
            "GetCommitHistory" => Some(GitSubject::query(QueryAction::GetCommitHistory)),
            "GetBranch" => Some(GitSubject::query(QueryAction::GetBranch)),
            "ListBranches" => Some(GitSubject::query(QueryAction::ListBranches)),
            "GetTag" => Some(GitSubject::query(QueryAction::GetTag)),
            "ListTags" => Some(GitSubject::query(QueryAction::ListTags)),
            "GetFileChanges" => Some(GitSubject::query(QueryAction::GetFileChanges)),
            _ => None,
        }
    }
}