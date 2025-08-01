// Copyright 2025 Cowboy AI, LLC.

//! CQRS adapter for Git domain handlers

use crate::commands::{CloneRepository, AnalyzeCommit, CreateBranch, DeleteBranch, CreateTag, AnalyzeRepository, FetchRemote, AnalyzeFileHistory, CompareBranches, SearchRepository, GitHubIntegration};
// TODO: ExtractCommitGraph and ExtractDependencyGraph have been removed
use crate::handlers::RepositoryCommandHandler;
use cim_domain::{
    CommandHandler, CommandEnvelope, CommandAcknowledgment, CommandStatus,
};

// TODO: ExtractCommitGraphHandler has been removed
// This was dependent on cim_domain_graph which is no longer available

/// CQRS adapter for `CloneRepository` command
pub struct CloneRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CloneRepositoryHandler {
    /// Create a new `CloneRepositoryHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CloneRepository> for CloneRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CloneRepository>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Clone repository using git2
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            // For now, analyze the repository at the given path
            // In a full implementation, this would actually clone from remote
            self.repository_handler.analyze_repository_at_path(&command.local_path).await
        });

        match result {
            Ok(_) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(format!("Failed to clone repository: {e}")),
            }
        }
    }
}

/// CQRS adapter for `AnalyzeCommit` command
pub struct AnalyzeCommitHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeCommitHandler {
    /// Create a new `AnalyzeCommitHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeCommit> for AnalyzeCommitHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeCommit>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Get repository and analyze specific commit
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would analyze the specific commit
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

// TODO: ExtractDependencyGraphHandler has been removed
// This was dependent on cim_domain_graph which is no longer available

/// CQRS adapter for `CreateBranch` command
pub struct CreateBranchHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CreateBranchHandler {
    /// Create a new `CreateBranchHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CreateBranch> for CreateBranchHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateBranch>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would create the branch
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `DeleteBranch` command
pub struct DeleteBranchHandler {
    repository_handler: RepositoryCommandHandler,
}

impl DeleteBranchHandler {
    /// Create a new `DeleteBranchHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<DeleteBranch> for DeleteBranchHandler {
    fn handle(&mut self, envelope: CommandEnvelope<DeleteBranch>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would delete the branch
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `CreateTag` command
pub struct CreateTagHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CreateTagHandler {
    /// Create a new `CreateTagHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CreateTag> for CreateTagHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateTag>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would create the tag
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `AnalyzeRepository` command
pub struct AnalyzeRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeRepositoryHandler {
    /// Create a new `AnalyzeRepositoryHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeRepository> for AnalyzeRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeRepository>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Get repository by ID
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(repository) => {
                if let Some(path) = repository.local_path {
                    // Analyze repository at the path
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    let result = runtime.block_on(async {
                        self.repository_handler.analyze_repository_at_path(&path).await
                    });

                    match result {
                        Ok(_) => CommandAcknowledgment {
                            command_id: envelope.id,
                            correlation_id: envelope.identity.correlation_id,
                            status: CommandStatus::Accepted,
                            reason: None,
                        },
                        Err(e) => CommandAcknowledgment {
                            command_id: envelope.id,
                            correlation_id: envelope.identity.correlation_id,
                            status: CommandStatus::Rejected,
                            reason: Some(format!("Failed to analyze repository: {e}")),
                        }
                    }
                } else {
                    CommandAcknowledgment {
                        command_id: envelope.id,
                        correlation_id: envelope.identity.correlation_id,
                        status: CommandStatus::Rejected,
                        reason: Some("Repository has no local path".to_string()),
                    }
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `FetchRemote` command
pub struct FetchRemoteHandler {
    repository_handler: RepositoryCommandHandler,
}

impl FetchRemoteHandler {
    /// Create a new `FetchRemoteHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<FetchRemote> for FetchRemoteHandler {
    fn handle(&mut self, envelope: CommandEnvelope<FetchRemote>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would fetch from remote
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `AnalyzeFileHistory` command
pub struct AnalyzeFileHistoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl AnalyzeFileHistoryHandler {
    /// Create a new `AnalyzeFileHistoryHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<AnalyzeFileHistory> for AnalyzeFileHistoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<AnalyzeFileHistory>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would analyze file history
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `CompareBranches` command
pub struct CompareBranchesHandler {
    repository_handler: RepositoryCommandHandler,
}

impl CompareBranchesHandler {
    /// Create a new `CompareBranchesHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<CompareBranches> for CompareBranchesHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CompareBranches>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would compare branches
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `SearchRepository` command
pub struct SearchRepositoryHandler {
    repository_handler: RepositoryCommandHandler,
}

impl SearchRepositoryHandler {
    /// Create a new `SearchRepositoryHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<SearchRepository> for SearchRepositoryHandler {
    fn handle(&mut self, envelope: CommandEnvelope<SearchRepository>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would search the repository
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
}

/// CQRS adapter for `GitHubIntegration` command
pub struct GitHubIntegrationHandler {
    repository_handler: RepositoryCommandHandler,
}

impl GitHubIntegrationHandler {
    /// Create a new `GitHubIntegrationHandler` with the given repository handler
    pub fn new(repository_handler: RepositoryCommandHandler) -> Self {
        Self { repository_handler }
    }
}

impl CommandHandler<GitHubIntegration> for GitHubIntegrationHandler {
    fn handle(&mut self, envelope: CommandEnvelope<GitHubIntegration>) -> CommandAcknowledgment {
        let command = envelope.command;
        
        // Check if repository exists
        let repo = self.repository_handler.get_repository(&command.repository_id);
        
        match repo {
            Some(_) => {
                // In a full implementation, would integrate with GitHub
                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.identity.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            None => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Repository not found".to_string()),
            }
        }
    }
} 