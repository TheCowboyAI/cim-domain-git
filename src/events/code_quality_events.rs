// Copyright 2025 Cowboy AI, LLC.

//! Code quality and health events for graph analytics
//!
//! These events track code quality metrics that can be used
//! to build technical debt and code health graphs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    aggregate::RepositoryId,
    value_objects::FilePath,
};

/// File complexity metrics calculated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileComplexityAnalyzed {
    /// Repository containing the file
    pub repository_id: RepositoryId,
    
    /// File analyzed
    pub file_path: FilePath,
    
    /// Cyclomatic complexity (if applicable)
    pub cyclomatic_complexity: Option<u32>,
    
    /// Lines of code
    pub lines_of_code: u32,
    
    /// Number of functions/methods
    pub function_count: u32,
    
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    
    /// Language detected
    pub language: String,
    
    /// When this analysis was performed
    pub timestamp: DateTime<Utc>,
}

/// File churn (frequency of changes) detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileChurnCalculated {
    /// Repository containing the file
    pub repository_id: RepositoryId,
    
    /// File with high churn
    pub file_path: FilePath,
    
    /// Number of changes in time window
    pub change_count: u32,
    
    /// Time window in days
    pub time_window_days: u32,
    
    /// Churn rate (changes per day)
    pub churn_rate: f64,
    
    /// Number of different authors
    pub unique_authors: u32,
    
    /// Risk level based on churn and complexity
    pub risk_level: RiskLevel,
    
    /// When this calculation was performed
    pub timestamp: DateTime<Utc>,
}

/// Risk levels for code health
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - stable code
    Low,
    /// Medium risk - some concerns
    Medium,
    /// High risk - needs attention
    High,
    /// Critical risk - immediate attention needed
    Critical,
}

/// Technical debt hotspot identified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalDebtIdentified {
    /// Repository with debt
    pub repository_id: RepositoryId,
    
    /// File or directory with debt
    pub path: FilePath,
    
    /// Type of debt detected
    pub debt_type: TechnicalDebtType,
    
    /// Severity score (0.0 to 1.0)
    pub severity: f64,
    
    /// Estimated effort to fix (in hours)
    pub estimated_effort_hours: Option<f64>,
    
    /// Evidence for this debt
    pub evidence: Vec<String>,
    
    /// When this debt was identified
    pub timestamp: DateTime<Utc>,
}

/// Types of technical debt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalDebtType {
    /// High complexity code
    HighComplexity,
    /// Frequently changing code (high churn)
    HighChurn,
    /// Large files that should be split
    LargeFile,
    /// Duplicated code detected
    Duplication,
    /// Poor test coverage
    LowTestCoverage,
    /// Many TODO/FIXME comments
    AccumulatedTodos,
    /// Outdated dependencies
    OutdatedDependencies,
    /// Inconsistent coding style
    InconsistentStyle,
}

/// Repository health metrics calculated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryHealthCalculated {
    /// Repository analyzed
    pub repository_id: RepositoryId,
    
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    
    /// Number of active contributors (last 90 days)
    pub active_contributors: u32,
    
    /// Commit frequency (commits per week)
    pub commit_frequency: f64,
    
    /// Percentage of stale branches
    pub stale_branch_percentage: f64,
    
    /// Average time to merge (in hours)
    pub average_merge_time_hours: Option<f64>,
    
    /// Code coverage percentage
    pub code_coverage: Option<f64>,
    
    /// Number of critical issues
    pub critical_issues: u32,
    
    /// When this calculation was performed
    pub timestamp: DateTime<Utc>,
}

/// Detected when files have circular dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircularDependencyDetected {
    /// Repository with circular dependency
    pub repository_id: RepositoryId,
    
    /// Files involved in the cycle
    pub cycle_files: Vec<FilePath>,
    
    /// Language/framework where detected
    pub language: String,
    
    /// When this was detected
    pub timestamp: DateTime<Utc>,
}