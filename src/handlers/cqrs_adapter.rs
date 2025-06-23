//! CQRS adapter for Git domain handlers

use crate::commands::*;
use crate::handlers::RepositoryCommandHandler;
use crate::GitDomainError;
use cim_domain::{
    CommandHandler, CommandEnvelope, CommandAcknowledgment, CommandStatus,
};

/// CQRS adapter for ExtractCommitGraph command
pub struct ExtractCommitGraphHandler {
    repository_handler: RepositoryCommandHandler,
}

impl ExtractCommitGraphHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<ExtractCommitGraph> for ExtractCommitGraphHandler {
    fn handle(&mut self, envelope: CommandEnvelope<ExtractCommitGraph>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Use tokio to run the async method
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            self.repository_handler.extract_commit_graph(command).await
        });

        match result {
            Ok(_events) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(format!("Failed to extract commit graph: {}", e)),
            }
        }
    }
}

/// CQRS adapter for CloneRepository command
pub struct CloneRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CloneRepositoryHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CloneRepository> for CloneRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CloneRepository>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would clone the repo
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for AnalyzeCommit command
pub struct AnalyzeCommitHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeCommitHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeCommit> for AnalyzeCommitHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeCommit>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would analyze the commit
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for ExtractDependencyGraph command
pub struct ExtractDependencyGraphHandler {
    repository_handler: RepositoryCommandHandler,
}

impl ExtractDependencyGraphHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<ExtractDependencyGraph> for ExtractDependencyGraphHandler {
    fn handle(&mut self, envelope: CommandEnvelope<ExtractDependencyGraph>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would extract dependencies
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for CreateBranch command
pub struct CreateBranchHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CreateBranchHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CreateBranch> for CreateBranchHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateBranch>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would create the branch
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for DeleteBranch command
pub struct DeleteBranchHandler {
    repository_handler: RepositoryCommandHandler,
}

impl DeleteBranchHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<DeleteBranch> for DeleteBranchHandler {
    fn handle(&mut self, envelope: CommandEnvelope<DeleteBranch>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would delete the branch
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for CreateTag command
pub struct CreateTagHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CreateTagHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CreateTag> for CreateTagHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateTag>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would create the tag
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for AnalyzeRepository command
pub struct AnalyzeRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeRepositoryHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeRepository> for AnalyzeRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeRepository>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would analyze the repository
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for FetchRemote command
pub struct FetchRemoteHandler {
    repository_handler: RepositoryCommandHandler,
}

impl FetchRemoteHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<FetchRemote> for FetchRemoteHandler {
    fn handle(&mut self, envelope: CommandEnvelope<FetchRemote>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would fetch from remote
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for AnalyzeFileHistory command
pub struct AnalyzeFileHistoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeFileHistoryHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeFileHistory> for AnalyzeFileHistoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeFileHistory>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would analyze file history
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for CompareBranches command
pub struct CompareBranchesHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CompareBranchesHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CompareBranches> for CompareBranchesHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CompareBranches>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would compare branches
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for SearchRepository command
pub struct SearchRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl SearchRepositoryHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<SearchRepository> for SearchRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<SearchRepository>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would search the repository
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for GitHubIntegration command
pub struct GitHubIntegrationHandler {
    repository_handler: RepositoryCommandHandler,
}

impl GitHubIntegrationHandler {
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<GitHubIntegration> for GitHubIntegrationHandler {
    fn handle(&mut self, envelope: CommandEnvelope<GitHubIntegration>) -> CommandAcknowledgment {
        // For now, just acknowledge - full implementation would integrate with GitHub
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
} 