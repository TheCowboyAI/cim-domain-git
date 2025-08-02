// Copyright 2025 Cowboy AI, LLC.

//! Code quality analyzer
//!
//! Analyzes code quality metrics for building technical debt graphs.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::{
    aggregate::RepositoryId,
    events::code_quality_events::*,
    value_objects::{AuthorInfo, FilePath},
};

/// Metrics for a single file
#[derive(Debug, Clone)]
pub struct FileMetrics {
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

/// Analyzes code quality patterns in a repository
pub struct CodeQualityAnalyzer {
    /// Threshold for large files
    large_file_threshold: u32,
    
    /// Threshold for high complexity
    high_complexity_threshold: u32,
    
    /// Days to consider for churn calculation
    churn_window_days: u32,
}

impl CodeQualityAnalyzer {
    /// Create a new code quality analyzer
    pub fn new() -> Self {
        Self {
            large_file_threshold: 500,
            high_complexity_threshold: 10,
            churn_window_days: 90,
        }
    }
    
    /// Analyze file complexity
    pub fn analyze_file_complexity(
        &self,
        repository_id: RepositoryId,
        file_path: FilePath,
        metrics: FileMetrics,
    ) -> FileComplexityAnalyzed {
        FileComplexityAnalyzed {
            repository_id,
            file_path,
            cyclomatic_complexity: metrics.cyclomatic_complexity,
            lines_of_code: metrics.lines_of_code,
            function_count: metrics.function_count,
            max_nesting_depth: metrics.max_nesting_depth,
            language: metrics.language,
            timestamp: Utc::now(),
        }
    }
    
    /// Calculate file churn
    pub fn calculate_file_churn(
        &self,
        repository_id: RepositoryId,
        file_path: FilePath,
        commits: &[(AuthorInfo, DateTime<Utc>)],
        file_metrics: Option<&FileMetrics>,
    ) -> FileChurnCalculated {
        let now = Utc::now();
        let window_start = now - Duration::days(self.churn_window_days as i64);
        
        // Filter commits within window
        let recent_commits: Vec<_> = commits
            .iter()
            .filter(|(_, timestamp)| *timestamp >= window_start)
            .collect();
        
        let change_count = recent_commits.len() as u32;
        let churn_rate = change_count as f64 / self.churn_window_days as f64;
        
        // Count unique authors
        let unique_authors = recent_commits
            .iter()
            .map(|(author, _)| author)
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
        
        // Calculate risk level
        let risk_level = self.calculate_risk_level(
            churn_rate,
            file_metrics.and_then(|m| m.cyclomatic_complexity),
            file_metrics.map(|m| m.lines_of_code).unwrap_or(0),
        );
        
        FileChurnCalculated {
            repository_id,
            file_path,
            change_count,
            time_window_days: self.churn_window_days,
            churn_rate,
            unique_authors,
            risk_level,
            timestamp: now,
        }
    }
    
    /// Identify technical debt hotspots
    pub fn identify_technical_debt(
        &self,
        repository_id: RepositoryId,
        file_analysis: Vec<(FilePath, FileMetrics, u32)>, // (path, metrics, change_count)
    ) -> Vec<TechnicalDebtIdentified> {
        let mut debts = Vec::new();
        
        for (path, metrics, change_count) in file_analysis {
            // Check for high complexity
            if let Some(complexity) = metrics.cyclomatic_complexity {
                if complexity > self.high_complexity_threshold {
                    let severity = (complexity as f64 / self.high_complexity_threshold as f64).min(1.0);
                    debts.push(TechnicalDebtIdentified {
                        repository_id: repository_id.clone(),
                        path: path.clone(),
                        debt_type: TechnicalDebtType::HighComplexity,
                        severity,
                        estimated_effort_hours: Some((complexity as f64 * 0.5).round()),
                        evidence: vec![
                            format!("Cyclomatic complexity: {}", complexity),
                            format!("Threshold: {}", self.high_complexity_threshold),
                        ],
                        timestamp: Utc::now(),
                    });
                }
            }
            
            // Check for large files
            if metrics.lines_of_code > self.large_file_threshold {
                let severity = (metrics.lines_of_code as f64 / self.large_file_threshold as f64).min(1.0);
                debts.push(TechnicalDebtIdentified {
                    repository_id: repository_id.clone(),
                    path: path.clone(),
                    debt_type: TechnicalDebtType::LargeFile,
                    severity,
                    estimated_effort_hours: Some((metrics.lines_of_code as f64 * 0.01).round()),
                    evidence: vec![
                        format!("Lines of code: {}", metrics.lines_of_code),
                        format!("Threshold: {}", self.large_file_threshold),
                    ],
                    timestamp: Utc::now(),
                });
            }
            
            // Check for high churn
            let churn_rate = change_count as f64 / self.churn_window_days as f64;
            if churn_rate > 0.5 {
                let severity = (churn_rate / 1.0).min(1.0);
                debts.push(TechnicalDebtIdentified {
                    repository_id: repository_id.clone(),
                    path: path.clone(),
                    debt_type: TechnicalDebtType::HighChurn,
                    severity,
                    estimated_effort_hours: Some(10.0),
                    evidence: vec![
                        format!("Changes in {} days: {}", self.churn_window_days, change_count),
                        format!("Churn rate: {:.2} changes/day", churn_rate),
                    ],
                    timestamp: Utc::now(),
                });
            }
        }
        
        debts
    }
    
    /// Calculate repository health
    pub fn calculate_repository_health(
        &self,
        repository_id: RepositoryId,
        active_contributors: u32,
        commits_last_week: u32,
        total_branches: u32,
        stale_branches: u32,
        critical_issues: u32,
    ) -> RepositoryHealthCalculated {
        // Calculate health score (0.0 to 1.0)
        let mut health_score = 1.0;
        
        // Penalize for few active contributors
        if active_contributors < 3 {
            health_score -= 0.2;
        }
        
        // Penalize for low commit frequency
        let commit_frequency = commits_last_week as f64;
        if commit_frequency < 5.0 {
            health_score -= 0.1;
        }
        
        // Penalize for stale branches
        let stale_percentage = if total_branches > 0 {
            stale_branches as f64 / total_branches as f64
        } else {
            0.0
        };
        health_score -= stale_percentage * 0.3;
        
        // Penalize for critical issues
        health_score -= (critical_issues as f64 * 0.1).min(0.4);
        
        health_score = health_score.max(0.0);
        
        RepositoryHealthCalculated {
            repository_id,
            health_score,
            active_contributors,
            commit_frequency,
            stale_branch_percentage: stale_percentage,
            average_merge_time_hours: None, // Would need PR data
            code_coverage: None, // Would need test data
            critical_issues,
            timestamp: Utc::now(),
        }
    }
    
    /// Detect circular dependencies
    pub fn detect_circular_dependencies(
        &self,
        repository_id: RepositoryId,
        language: String,
        file_dependencies: &HashMap<FilePath, Vec<FilePath>>,
    ) -> Vec<CircularDependencyDetected> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        for (file, _) in file_dependencies {
            if !visited.contains(file) {
                if let Some(cycle) = self.find_cycle(
                    file,
                    file_dependencies,
                    &mut visited,
                    &mut path,
                    &mut std::collections::HashSet::new(),
                ) {
                    cycles.push(CircularDependencyDetected {
                        repository_id: repository_id.clone(),
                        cycle_files: cycle,
                        language: language.clone(),
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        cycles
    }
    
    // Helper methods
    
    fn calculate_risk_level(
        &self,
        churn_rate: f64,
        complexity: Option<u32>,
        lines_of_code: u32,
    ) -> RiskLevel {
        let mut risk_score = 0.0;
        
        // Churn contributes to risk
        risk_score += churn_rate * 2.0;
        
        // Complexity contributes to risk
        if let Some(c) = complexity {
            risk_score += c as f64 / self.high_complexity_threshold as f64;
        }
        
        // Size contributes to risk
        risk_score += lines_of_code as f64 / self.large_file_threshold as f64 * 0.5;
        
        match risk_score {
            x if x >= 3.0 => RiskLevel::Critical,
            x if x >= 2.0 => RiskLevel::High,
            x if x >= 1.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
    
    fn find_cycle(
        &self,
        current: &FilePath,
        graph: &HashMap<FilePath, Vec<FilePath>>,
        visited: &mut std::collections::HashSet<FilePath>,
        path: &mut Vec<FilePath>,
        rec_stack: &mut std::collections::HashSet<FilePath>,
    ) -> Option<Vec<FilePath>> {
        visited.insert(current.clone());
        rec_stack.insert(current.clone());
        path.push(current.clone());
        
        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.find_cycle(neighbor, graph, visited, path, rec_stack) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|p| p == neighbor) {
                        return Some(path[start_idx..].to_vec());
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(current);
        None
    }
}

impl Default for CodeQualityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_level_calculation() {
        let analyzer = CodeQualityAnalyzer::new();
        let repo_id = RepositoryId::new();
        let file_path = FilePath::new("src/main.rs").unwrap();
        
        // High churn should produce high risk
        // Create many commits to simulate high churn
        let now = Utc::now();
        let mut commits = Vec::new();
        for i in 0..50 {
            commits.push((
                AuthorInfo { name: format!("Dev{}", i % 3), email: format!("dev{}@example.com", i % 3) },
                now - Duration::days(i as i64),
            ));
        }
        
        let churn_result = analyzer.calculate_file_churn(
            repo_id.clone(),
            file_path.clone(),
            &commits,
            Some(&FileMetrics {
                lines_of_code: 1000,
                function_count: 20,
                max_nesting_depth: 5,
                cyclomatic_complexity: Some(25),
                language: "rust".to_string(),
            }),
        );
        
        // With 50 commits in 90 days (0.55 changes/day) + high complexity, should be High risk
        assert!(matches!(churn_result.risk_level, RiskLevel::High | RiskLevel::Critical));
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let analyzer = CodeQualityAnalyzer::new();
        let repo_id = RepositoryId::new();
        
        let mut deps = HashMap::new();
        deps.insert(
            FilePath::new("a.rs").unwrap(),
            vec![FilePath::new("b.rs").unwrap()],
        );
        deps.insert(
            FilePath::new("b.rs").unwrap(),
            vec![FilePath::new("c.rs").unwrap()],
        );
        deps.insert(
            FilePath::new("c.rs").unwrap(),
            vec![FilePath::new("a.rs").unwrap()],
        );
        
        let cycles = analyzer.detect_circular_dependencies(
            repo_id,
            "rust".to_string(),
            &deps,
        );
        
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].cycle_files.len(), 3);
    }
}