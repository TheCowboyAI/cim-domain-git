// Copyright 2025 Cowboy AI, LLC.

//! Example demonstrating graph-focused analysis of Git repositories
//!
//! This example shows how to use the collaboration and code quality analyzers
//! to extract metadata optimized for building graphs with cim-ipld and cim-domain-graphs.

use chrono::Utc;
use cim_domain_git::{
    aggregate::RepositoryId,
    analyzers::{CollaborationAnalyzer, CodeQualityAnalyzer, FileMetrics},
    value_objects::{AuthorInfo, CommitHash, FilePath},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize analyzers
    let collaboration_analyzer = CollaborationAnalyzer::new();
    let code_quality_analyzer = CodeQualityAnalyzer::new();
    
    // Sample repository data
    let repo_id = RepositoryId::new();
    
    // Sample commit data
    let commits = vec![
        (
            CommitHash::new("abc123def456789").unwrap(),
            AuthorInfo::new("Alice".to_string(), "alice@example.com".to_string()),
            vec![
                FilePath::new("src/main.rs").unwrap(),
                FilePath::new("src/lib.rs").unwrap(),
            ],
            Utc::now(),
        ),
        (
            CommitHash::new("def456abc789012").unwrap(),
            AuthorInfo::new("Bob".to_string(), "bob@example.com".to_string()),
            vec![
                FilePath::new("src/main.rs").unwrap(),
                FilePath::new("src/utils.rs").unwrap(),
            ],
            Utc::now(),
        ),
        (
            CommitHash::new("789012def345678").unwrap(),
            AuthorInfo::new("Alice".to_string(), "alice@example.com".to_string()),
            vec![
                FilePath::new("src/lib.rs").unwrap(),
                FilePath::new("src/utils.rs").unwrap(),
            ],
            Utc::now(),
        ),
    ];
    
    // Analyze collaboration patterns
    println!("=== Collaboration Analysis ===");
    let collaborations = collaboration_analyzer.analyze_collaboration(
        repo_id.clone(),
        &commits,
    );
    
    for collab in &collaborations {
        println!("Collaboration detected:");
        println!("  Authors: {:?}", collab.authors);
        println!("  Shared files: {:?}", collab.shared_files);
        println!("  Collaboration strength: {:.2}", collab.collaboration_strength);
        println!();
    }
    
    // Calculate code ownership
    println!("=== Code Ownership Analysis ===");
    let mut file_commits: HashMap<FilePath, Vec<(AuthorInfo, chrono::DateTime<chrono::Utc>)>> = HashMap::new();
    for (_, author, files, timestamp) in &commits {
        for file in files {
            file_commits
                .entry(file.clone())
                .or_insert_with(Vec::new)
                .push((author.clone(), *timestamp));
        }
    }
    
    let ownership_events = collaboration_analyzer.calculate_ownership(
        repo_id.clone(),
        &file_commits,
    );
    
    for ownership in ownership_events {
        println!("File ownership:");
        println!("  Path: {}", ownership.path);
        println!("  Primary owner: {:?}", ownership.primary_owner);
        println!("  Ownership percentage: {:.2}%", ownership.ownership_percentage * 100.0);
        println!("  Contributors: {:?}", ownership.contributors);
        println!();
    }
    
    // Detect team clusters
    println!("=== Team Cluster Detection ===");
    let team_clusters = collaboration_analyzer.detect_team_clusters(
        repo_id.clone(),
        &collaborations,
        2, // min team size
    );
    
    for team in team_clusters {
        println!("Team cluster detected:");
        println!("  Members: {:?}", team.team_members);
        println!("  Cohesion score: {:.2}", team.cohesion_score);
        println!("  Focus areas: {:?}", team.focus_areas);
        println!();
    }
    
    // Analyze code quality
    println!("=== Code Quality Analysis ===");
    
    // Sample file metrics
    let file_metrics = vec![
        (
            FilePath::new("src/main.rs").unwrap(),
            FileMetrics {
                lines_of_code: 500,
                function_count: 20,
                max_nesting_depth: 5,
                cyclomatic_complexity: Some(25),
                language: "rust".to_string(),
            },
        ),
        (
            FilePath::new("src/lib.rs").unwrap(),
            FileMetrics {
                lines_of_code: 1200,
                function_count: 50,
                max_nesting_depth: 7,
                cyclomatic_complexity: Some(40),
                language: "rust".to_string(),
            },
        ),
    ];
    
    // Analyze file complexity
    for (path, metrics) in &file_metrics {
        let complexity_event = code_quality_analyzer.analyze_file_complexity(
            repo_id.clone(),
            path.clone(),
            metrics.clone(),
        );
        
        println!("File complexity analysis:");
        println!("  Path: {}", complexity_event.file_path);
        println!("  Lines of code: {}", complexity_event.lines_of_code);
        println!("  Cyclomatic complexity: {:?}", complexity_event.cyclomatic_complexity);
        println!("  Language: {}", complexity_event.language);
        println!();
    }
    
    // Calculate file churn
    for (path, commits) in &file_commits {
        let churn_event = code_quality_analyzer.calculate_file_churn(
            repo_id.clone(),
            path.clone(),
            commits,
            file_metrics.iter()
                .find(|(p, _)| p == path)
                .map(|(_, m)| m),
        );
        
        println!("File churn analysis:");
        println!("  Path: {}", churn_event.file_path);
        println!("  Change count: {}", churn_event.change_count);
        println!("  Churn rate: {:.2} changes/day", churn_event.churn_rate);
        println!("  Risk level: {:?}", churn_event.risk_level);
        println!();
    }
    
    // Identify technical debt
    println!("=== Technical Debt Analysis ===");
    let file_analysis: Vec<_> = file_metrics
        .iter()
        .map(|(path, metrics)| {
            let change_count = file_commits.get(path).map(|c| c.len()).unwrap_or(0) as u32;
            (path.clone(), metrics.clone(), change_count)
        })
        .collect();
    
    let debt_events = code_quality_analyzer.identify_technical_debt(
        repo_id.clone(),
        file_analysis,
    );
    
    for debt in debt_events {
        println!("Technical debt identified:");
        println!("  Path: {}", debt.path);
        println!("  Type: {:?}", debt.debt_type);
        println!("  Severity: {:.2}", debt.severity);
        println!("  Estimated effort: {:?} hours", debt.estimated_effort_hours);
        println!("  Evidence: {:?}", debt.evidence);
        println!();
    }
    
    // Calculate repository health
    println!("=== Repository Health ===");
    let health_event = code_quality_analyzer.calculate_repository_health(
        repo_id.clone(),
        2,  // active contributors
        10, // commits last week
        5,  // total branches
        1,  // stale branches
        0,  // critical issues
    );
    
    println!("Repository health score: {:.2}", health_event.health_score);
    println!("Active contributors: {}", health_event.active_contributors);
    println!("Commit frequency: {:.2} commits/week", health_event.commit_frequency);
    println!("Stale branch percentage: {:.2}%", health_event.stale_branch_percentage * 100.0);
    
    println!("\n=== Summary ===");
    println!("This analysis provides graph-ready metadata for:");
    println!("- Social collaboration graphs");
    println!("- Code ownership networks");
    println!("- Technical debt heatmaps");
    println!("- Repository health dashboards");
    println!("\nThis data can be streamed to cim-ipld and cim-domain-graphs");
    println!("for building interactive visualizations and analytics.");
    
    Ok(())
}