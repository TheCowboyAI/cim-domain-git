// Copyright 2025 Cowboy AI, LLC.

//! Command for analyzing repository for graph building

use cim_domain::DomainCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    aggregate::RepositoryId,
    value_objects::{AuthorInfo, CommitHash, FilePath},
};

/// Command to analyze repository for graph building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeForGraphs {
    /// Repository to analyze
    pub repository_id: RepositoryId,
    
    /// Correlation ID for tracking
    pub correlation_id: String,
    
    /// Command ID for causation tracking
    pub command_id: String,
    
    /// Commits to analyze: (hash, author, files, timestamp)
    pub commits: Vec<(CommitHash, AuthorInfo, Vec<FilePath>, chrono::DateTime<chrono::Utc>)>,
    
    /// File metrics for quality analysis
    pub file_metrics: HashMap<FilePath, FileMetricsInput>,
    
    /// Repository health metrics
    pub repository_health_metrics: Option<RepositoryHealthMetrics>,
    
    /// Whether to analyze collaboration patterns
    pub analyze_collaboration: bool,
    
    /// Whether to analyze code quality
    pub analyze_code_quality: bool,
    
    /// Minimum team size for cluster detection
    pub min_team_size: Option<usize>,
}

/// Input for file metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetricsInput {
    /// Lines of code
    pub lines_of_code: u32,
    
    /// Number of functions/methods
    pub function_count: u32,
    
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    
    /// Cyclomatic complexity (if available)
    pub cyclomatic_complexity: Option<u32>,
    
    /// Language detected
    pub language: String,
}

/// Repository health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealthMetrics {
    /// Number of active contributors (last 90 days)
    pub active_contributors: u32,
    
    /// Commits in the last week
    pub commits_last_week: u32,
    
    /// Total number of branches
    pub total_branches: u32,
    
    /// Number of stale branches
    pub stale_branches: u32,
    
    /// Number of critical issues
    pub critical_issues: u32,
}

impl DomainCommand for AnalyzeForGraphs {
    fn command_type(&self) -> &'static str {
        "AnalyzeForGraphs"
    }
    
    fn aggregate_id(&self) -> String {
        self.repository_id.to_string()
    }
}