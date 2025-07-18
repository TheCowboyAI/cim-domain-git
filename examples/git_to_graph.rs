//! Git to Graph Conversion Example
//!
//! This example demonstrates how to:
//! 1. Analyze the current Git repository using the Git domain
//! 2. Extract commit graphs and dependency graphs
//! 3. Convert Git structures to graph domain entities
//! 4. Visualize the resulting graphs
//!
//! ```mermaid
//! graph TD
//!     A[Current Git Repo] --> B[Git Domain Analysis]
//!     B --> C[Extract Commit Graph]
//!     B --> D[Extract Dependency Graph]
//!     C --> E[Graph Domain Entities]
//!     D --> E
//!     E --> F[Visualize Graph Structure]
//!     E --> G[Export Graph Data]
//! ```
//!
//! ## Usage
//!
//! Run this example from the root of any Git repository:
//!
//! ```bash
//! cd /path/to/your/git/repo
//! cargo run --example git_to_graph
//! ```

use cim_domain_git::{
    GitDomainError,
    commands::{ExtractCommitGraph, ExtractDependencyGraph},
    handlers::RepositoryCommandHandler,
};
use cim_domain_graph::{
    commands::{GraphCommand, GraphCommandError},
    domain_events::GraphDomainEvent,
    handlers::{GraphCommandHandler, GraphCommandHandlerImpl, InMemoryGraphRepository},
};
use std::sync::Arc;
use tracing::{info, warn};

/// Git to Graph converter that integrates Git and Graph domains
pub struct GitToGraphConverter {
    git_handler: RepositoryCommandHandler,
    graph_handler: GraphCommandHandlerImpl,
}

impl GitToGraphConverter {
    /// Create a new converter
    pub fn new() -> Self {
        let repository = Arc::new(InMemoryGraphRepository::new());
        Self {
            git_handler: RepositoryCommandHandler::new(),
            graph_handler: GraphCommandHandlerImpl::new(repository),
        }
    }

    /// Analyze the current repository and convert to graphs
    pub async fn analyze_and_convert(&self) -> Result<GitAnalysisResult, ConverterError> {
        info!("Starting Git repository analysis and graph conversion");

        // Step 1: Analyze the current Git repository
        let (repository_id, git_events) = self
            .git_handler
            .analyze_current_repository()
            .await
            .map_err(ConverterError::GitError)?;

        info!("Repository analyzed: {} events generated", git_events.len());

        // Step 2: Create graphs for different aspects of the repository
        let mut result = GitAnalysisResult {
            repository_id,
            commit_graph_id: None,
            dependency_graph_id: None,
            graphs_created: 0,
            analysis_summary: AnalysisSummary::default(),
        };

        // Extract commit graph
        match self.extract_commit_graph(repository_id).await {
            Ok(graph_id) => {
                result.commit_graph_id = Some(graph_id);
                result.graphs_created += 1;
                info!("Commit graph created: {:?}", graph_id);
            }
            Err(e) => {
                warn!("Failed to create commit graph: {}", e);
            }
        }

        // Extract dependency graph
        match self.extract_dependency_graph(repository_id).await {
            Ok(graph_id) => {
                result.dependency_graph_id = Some(graph_id);
                result.graphs_created += 1;
                info!("Dependency graph created: {:?}", graph_id);
            }
            Err(e) => {
                warn!("Failed to create dependency graph: {}", e);
            }
        }

        // Collect analysis summary
        result.analysis_summary = self.collect_analysis_summary(&git_events);

        info!(
            "Git to Graph conversion completed: {} graphs created",
            result.graphs_created
        );

        Ok(result)
    }

    /// Extract commit graph from the repository
    async fn extract_commit_graph(
        &self,
        repository_id: cim_domain_git::aggregate::RepositoryId,
    ) -> Result<cim_domain_graph::GraphId, ConverterError> {
        info!("Extracting commit graph from repository");

        // Create Git domain command to extract commit graph
        let extract_cmd = ExtractCommitGraph {
            repository_id,
            start_commit: None,  // Start from HEAD
            max_depth: Some(50), // Limit to 50 commits for demo
            include_all_branches: true,
            include_tags: true,
        };

        // Execute Git domain command
        let git_events = self
            .git_handler
            .extract_commit_graph(extract_cmd)
            .await
            .map_err(ConverterError::GitError)?;

        // Find the commit graph extracted event
        let commit_graph_event = git_events
            .iter()
            .find_map(|event| {
                if let cim_domain_git::events::GitDomainEvent::CommitGraphExtracted(e) = event {
                    Some(e)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                ConverterError::ConversionError("No commit graph extracted".to_string())
            })?;

        // Create corresponding graph in Graph domain
        let create_graph_cmd = GraphCommand::CreateGraph {
            name: format!("Commit Graph - Repository {repository_id.as_uuid(}")),
            description: format!("Git commit history graph with {commit_graph_event.commit_count} commits and {commit_graph_event.edge_count} edges"),
            metadata: {
                let mut data = std::collections::HashMap::new();
                data.insert(
                    "repository_id".to_string(),
                    serde_json::Value::String(repository_id.as_uuid().to_string()),
                );
                data.insert(
                    "source".to_string(),
                    serde_json::Value::String("git_commit_graph".to_string()),
                );
                data.insert(
                    "commit_count".to_string(),
                    serde_json::Value::Number(commit_graph_event.commit_count.into()),
                );
                data.insert(
                    "created_by".to_string(),
                    serde_json::Value::String("GitToGraphConverter".to_string()),
                );
                data
            },
        };

        let graph_events = self
            .graph_handler
            .handle_graph_command(create_graph_cmd)
            .await
            .map_err(ConverterError::GraphError)?;

        // Extract graph ID from events
        let graph_id = graph_events
            .iter()
            .find_map(|event| {
                if let GraphDomainEvent::GraphCreated(e) = event {
                    Some(e.graph_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| ConverterError::ConversionError("Graph not created".to_string()))?;

        info!("Commit graph created in Graph domain: {:?}", graph_id);
        Ok(graph_id)
    }

    /// Extract dependency graph from the repository
    async fn extract_dependency_graph(
        &self,
        repository_id: cim_domain_git::aggregate::RepositoryId,
    ) -> Result<cim_domain_graph::GraphId, ConverterError> {
        info!("Extracting dependency graph from repository");

        // Create Git domain command to extract dependency graph
        let _extract_cmd = ExtractDependencyGraph {
            repository_id,
            commit_hash: None, // Use HEAD
            include_patterns: vec![
                "*.rs".to_string(),
                "*.toml".to_string(),
                "*.nix".to_string(),
                "*.md".to_string(),
            ],
            exclude_patterns: vec![
                "target/".to_string(),
                ".git/".to_string(),
                "*.lock".to_string(),
            ],
            language: Some("rust".to_string()),
        };

        // Execute Git domain command (simplified - the real implementation would need access to git2)
        // For now, we'll create a basic dependency graph structure
        // Note: _extract_cmd will be used in a full implementation

        let create_graph_cmd = GraphCommand::CreateGraph {
            name: format!("Dependency Graph - Repository {repository_id.as_uuid(}")),
            description: "File dependency relationships extracted from Git repository".to_string(),
            metadata: {
                let mut data = std::collections::HashMap::new();
                data.insert(
                    "repository_id".to_string(),
                    serde_json::Value::String(repository_id.as_uuid().to_string()),
                );
                data.insert(
                    "source".to_string(),
                    serde_json::Value::String("git_dependency_graph".to_string()),
                );
                data.insert(
                    "analysis_type".to_string(),
                    serde_json::Value::String("file_dependencies".to_string()),
                );
                data.insert(
                    "created_by".to_string(),
                    serde_json::Value::String("GitToGraphConverter".to_string()),
                );
                data
            },
        };

        let graph_events = self
            .graph_handler
            .handle_graph_command(create_graph_cmd)
            .await
            .map_err(ConverterError::GraphError)?;

        // Extract graph ID from events
        let graph_id = graph_events
            .iter()
            .find_map(|event| {
                if let GraphDomainEvent::GraphCreated(e) = event {
                    Some(e.graph_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| ConverterError::ConversionError("Graph not created".to_string()))?;

        info!("Dependency graph created in Graph domain: {:?}", graph_id);
        Ok(graph_id)
    }

    /// Collect analysis summary from Git events
    fn collect_analysis_summary(
        &self,
        events: &[cim_domain_git::events::GitDomainEvent],
    ) -> AnalysisSummary {
        let mut summary = AnalysisSummary::default();

        for event in events {
            match event {
                cim_domain_git::events::GitDomainEvent::RepositoryAnalyzed(e) => {
                    summary.repository_name = e.name.clone();
                    summary.total_branches = e.branch_count;
                    summary.total_commits = e.commit_count;
                }
                cim_domain_git::events::GitDomainEvent::BranchCreated(e) => {
                    summary
                        .branches_analyzed
                        .push(e.branch_name.as_str().to_string());
                }
                cim_domain_git::events::GitDomainEvent::CommitAnalyzed(e) => {
                    summary
                        .commits_analyzed
                        .push(e.commit_hash.as_str().to_string());
                    summary.unique_authors.insert(e.author.name.clone());
                }
                _ => {}
            }
        }

        summary
    }

    /// Display the analysis results
    pub fn display_results(&self, result: &GitAnalysisResult) {
        println!("\n=== Git Repository Analysis Results ===");
        println!("Repository ID: {result.repository_id.as_uuid(}"));
        println!("Repository Name: {result.analysis_summary.repository_name}");
        println!("Graphs Created: {result.graphs_created}");

        if let Some(commit_graph_id) = result.commit_graph_id {
            println!("Commit Graph ID: {commit_graph_id.as_uuid(}"));
        }

        if let Some(dependency_graph_id) = result.dependency_graph_id {
            println!("Dependency Graph ID: {dependency_graph_id.as_uuid(}"));
        }

        println!("\n=== Analysis Summary ===");
        println!("Total Branches: {result.analysis_summary.total_branches}");
        println!("Total Commits: {result.analysis_summary.total_commits}");
        println!("Commits Analyzed: {result.analysis_summary.commits_analyzed.len(}")
        );
        println!("Unique Authors: {result.analysis_summary.unique_authors.len(}")
        );

        if !result.analysis_summary.branches_analyzed.is_empty() {
            println!("Branches Found:");
            for branch in &result.analysis_summary.branches_analyzed {
                println!("  - {branch}");
            }
        }

        if !result.analysis_summary.unique_authors.is_empty() {
            println!("Authors Found:");
            for author in &result.analysis_summary.unique_authors {
                println!("  - {author}");
            }
        }
    }
}

impl Default for GitToGraphConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of Git analysis and graph conversion
#[derive(Debug)]
pub struct GitAnalysisResult {
    /// Repository ID in Git domain
    pub repository_id: cim_domain_git::aggregate::RepositoryId,
    /// Commit graph ID in Graph domain
    pub commit_graph_id: Option<cim_domain_graph::GraphId>,
    /// Dependency graph ID in Graph domain
    pub dependency_graph_id: Option<cim_domain_graph::GraphId>,
    /// Number of graphs successfully created
    pub graphs_created: usize,
    /// Summary of analysis
    pub analysis_summary: AnalysisSummary,
}

/// Summary of Git repository analysis
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    pub repository_name: String,
    pub total_branches: usize,
    pub total_commits: usize,
    pub branches_analyzed: Vec<String>,
    pub commits_analyzed: Vec<String>,
    pub unique_authors: std::collections::HashSet<String>,
}

/// Converter-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ConverterError {
    #[error("Git domain error: {0}")]
    GitError(#[from] GitDomainError),

    #[error("Graph domain error: {0}")]
    GraphError(#[from] GraphCommandError),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Main example function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Git to Graph Conversion Example");
    println!("This example analyzes the current Git repository and creates graph representations");

    // Check if we're in a Git repository
    if !std::path::Path::new(".git").exists() {
        eprintln!(
            "❌ Error: Not in a Git repository. Please run this example from the root of a Git repository."
        );
        std::process::exit(1);
    }

    // Create converter
    let converter = GitToGraphConverter::new();

    // Perform analysis and conversion
    match converter.analyze_and_convert().await {
        Ok(result) => {
            converter.display_results(&result);

            if result.graphs_created > 0 {
                println!("\n✅ Success! Created {result.graphs_created} graph(s) from Git repository data");
                println!("\nNext steps:");
                println!("1. Use the Graph domain queries to explore the created graphs");
                println!("2. Visualize the graphs using the Bevy presentation layer");
                println!("3. Apply graph algorithms to analyze repository structure");
            } else {
                println!("\n⚠️  No graphs were created. Check the logs for details.");
            }
        }
        Err(e) => {
            eprintln!("❌ Error during Git to Graph conversion: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_converter_creation() {
        let converter = GitToGraphConverter::new();
        // Just test that we can create the converter without panicking
        assert_eq!(converter.git_handler.list_repositories().len(), 0);
    }

    #[test]
    fn test_analysis_summary() {
        let summary = AnalysisSummary {
            repository_name: "test-repo".to_string(),
            total_branches: 3,
            total_commits: 15,
            branches_analyzed: vec!["main".to_string(), "dev".to_string()],
            commits_analyzed: vec!["abc123".to_string()],
            unique_authors: vec!["Alice".to_string(), "Bob".to_string()]
                .into_iter()
                .collect(),
        };

        assert_eq!(summary.repository_name, "test-repo");
        assert_eq!(summary.total_branches, 3);
        assert_eq!(summary.unique_authors.len(), 2);
    }
}
