// Copyright 2025 Cowboy AI, LLC.

//! Collaboration pattern analyzer
//!
//! Analyzes git history to detect collaboration patterns between developers,
//! useful for building social graphs and team dynamics visualizations.

use chrono::{DateTime, Utc};
#[cfg(test)]
use chrono::Duration;
use std::collections::{HashMap, HashSet};

use crate::{
    aggregate::RepositoryId,
    events::collaboration_events::*,
    value_objects::{AuthorInfo, CommitHash, FilePath},
};

/// Analyzes collaboration patterns in a repository
pub struct CollaborationAnalyzer {
    /// Time window for considering commits as collaborative
    collaboration_window_hours: i64,
    
    /// Minimum shared files to consider collaboration
    min_shared_files: usize,
}

impl CollaborationAnalyzer {
    /// Create a new collaboration analyzer
    pub fn new() -> Self {
        Self {
            collaboration_window_hours: 168, // 1 week
            min_shared_files: 2,
        }
    }
    
    /// Analyze commits to find collaboration patterns
    pub fn analyze_collaboration(
        &self,
        repository_id: RepositoryId,
        commits: &[(CommitHash, AuthorInfo, Vec<FilePath>, DateTime<Utc>)],
    ) -> Vec<CollaborationDetected> {
        let mut collaborations = Vec::new();
        let mut author_files: HashMap<AuthorInfo, HashMap<FilePath, Vec<DateTime<Utc>>>> = HashMap::new();
        
        // Build author -> file -> timestamps map
        for (_, author, files, timestamp) in commits {
            let file_map = author_files.entry(author.clone()).or_insert_with(HashMap::new);
            for file in files {
                file_map.entry(file.clone()).or_insert_with(Vec::new).push(*timestamp);
            }
        }
        
        // Find collaborations between author pairs
        let authors: Vec<_> = author_files.keys().cloned().collect();
        for i in 0..authors.len() {
            for j in (i + 1)..authors.len() {
                let author1 = &authors[i];
                let author2 = &authors[j];
                
                let files1 = &author_files[author1];
                let files2 = &author_files[author2];
                
                // Find shared files
                let mut shared_files = Vec::new();
                let collaboration_strength;
                let mut time_overlaps = 0;
                
                for (file, timestamps1) in files1 {
                    if let Some(timestamps2) = files2.get(file) {
                        shared_files.push(file.clone());
                        
                        // Check for time proximity
                        for t1 in timestamps1 {
                            for t2 in timestamps2 {
                                let diff = (*t1 - *t2).num_hours().abs();
                                if diff <= self.collaboration_window_hours {
                                    time_overlaps += 1;
                                }
                            }
                        }
                    }
                }
                
                if shared_files.len() >= self.min_shared_files {
                    // Calculate collaboration strength
                    let total_files = files1.len() + files2.len();
                    let shared_ratio = (shared_files.len() * 2) as f64 / total_files as f64;
                    let time_factor = (time_overlaps as f64 / shared_files.len() as f64).min(1.0);
                    collaboration_strength = shared_ratio * 0.6 + time_factor * 0.4;
                    
                    collaborations.push(CollaborationDetected {
                        repository_id: repository_id.clone(),
                        authors: vec![author1.clone(), author2.clone()],
                        shared_files: shared_files.clone(),
                        time_window_hours: self.collaboration_window_hours as u32,
                        collaboration_strength,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        collaborations
    }
    
    /// Detect code ownership patterns
    pub fn calculate_ownership(
        &self,
        repository_id: RepositoryId,
        file_commits: &HashMap<FilePath, Vec<(AuthorInfo, DateTime<Utc>)>>,
    ) -> Vec<CodeOwnershipCalculated> {
        let mut ownership_events = Vec::new();
        
        for (file_path, commits) in file_commits {
            let mut author_counts: HashMap<AuthorInfo, u32> = HashMap::new();
            
            // Count commits per author
            for (author, _) in commits {
                *author_counts.entry(author.clone()).or_insert(0) += 1;
            }
            
            let total_commits = commits.len() as u32;
            if total_commits == 0 {
                continue;
            }
            
            // Find primary owner
            let (primary_owner, primary_count) = author_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(author, count)| (author.clone(), *count))
                .unwrap();
            
            let ownership_percentage = primary_count as f64 / total_commits as f64;
            
            // Calculate other contributors
            let mut contributors: Vec<(AuthorInfo, f64)> = author_counts
                .into_iter()
                .filter(|(author, _)| author != &primary_owner)
                .map(|(author, count)| (author, count as f64 / total_commits as f64))
                .collect();
            
            contributors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            ownership_events.push(CodeOwnershipCalculated {
                repository_id: repository_id.clone(),
                path: file_path.clone(),
                primary_owner,
                ownership_percentage,
                contributors,
                total_commits,
                timestamp: Utc::now(),
            });
        }
        
        ownership_events
    }
    
    /// Detect team clusters based on collaboration patterns
    pub fn detect_team_clusters(
        &self,
        repository_id: RepositoryId,
        collaborations: &[CollaborationDetected],
        min_team_size: usize,
    ) -> Vec<TeamClusterDetected> {
        let mut teams = Vec::new();
        let mut author_connections: HashMap<AuthorInfo, HashMap<AuthorInfo, f64>> = HashMap::new();
        
        // Build connection graph
        for collab in collaborations {
            if collab.authors.len() == 2 {
                let author1 = &collab.authors[0];
                let author2 = &collab.authors[1];
                
                author_connections
                    .entry(author1.clone())
                    .or_insert_with(HashMap::new)
                    .insert(author2.clone(), collab.collaboration_strength);
                
                author_connections
                    .entry(author2.clone())
                    .or_insert_with(HashMap::new)
                    .insert(author1.clone(), collab.collaboration_strength);
            }
        }
        
        // Find clusters using simple community detection
        let mut visited = HashSet::new();
        
        for (author, _connections) in &author_connections {
            if visited.contains(author) {
                continue;
            }
            
            // Find all authors strongly connected to this one
            let mut team_members = vec![author.clone()];
            let mut to_check = vec![author.clone()];
            visited.insert(author.clone());
            
            while let Some(current) = to_check.pop() {
                if let Some(current_connections) = author_connections.get(&current) {
                    for (connected_author, strength) in current_connections {
                        if *strength > 0.5 && !visited.contains(connected_author) {
                            team_members.push(connected_author.clone());
                            to_check.push(connected_author.clone());
                            visited.insert(connected_author.clone());
                        }
                    }
                }
            }
            
            if team_members.len() >= min_team_size {
                // Calculate cohesion score
                let mut internal_connections = 0;
                let mut external_connections = 0;
                
                for member in &team_members {
                    if let Some(connections) = author_connections.get(member) {
                        for (connected, _) in connections {
                            if team_members.contains(connected) {
                                internal_connections += 1;
                            } else {
                                external_connections += 1;
                            }
                        }
                    }
                }
                
                let cohesion_score = if internal_connections + external_connections > 0 {
                    internal_connections as f64 / (internal_connections + external_connections) as f64
                } else {
                    0.0
                };
                
                // Find focus areas (files this team works on)
                let mut team_files = HashSet::new();
                for collab in collaborations {
                    if collab.authors.iter().all(|a| team_members.contains(a)) {
                        team_files.extend(collab.shared_files.iter().cloned());
                    }
                }
                
                teams.push(TeamClusterDetected {
                    repository_id: repository_id.clone(),
                    team_members,
                    cohesion_score,
                    focus_areas: team_files.into_iter().collect(),
                    timestamp: Utc::now(),
                });
            }
        }
        
        teams
    }
}

impl Default for CollaborationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_author(name: &str) -> AuthorInfo {
        AuthorInfo {
            name: name.to_string(),
            email: format!("{}@example.com", name.to_lowercase()),
        }
    }
    
    #[test]
    fn test_collaboration_detection() {
        let analyzer = CollaborationAnalyzer::new();
        let repo_id = RepositoryId::new();
        
        let now = Utc::now();
        let commits = vec![
            (
                CommitHash::new("abc123def456789").unwrap(),
                create_test_author("Alice"),
                vec![FilePath::new("src/main.rs").unwrap(), FilePath::new("src/lib.rs").unwrap()],
                now,
            ),
            (
                CommitHash::new("def456abc789012").unwrap(),
                create_test_author("Bob"),
                vec![FilePath::new("src/main.rs").unwrap(), FilePath::new("src/lib.rs").unwrap()],
                now + Duration::hours(2),
            ),
        ];
        
        let collaborations = analyzer.analyze_collaboration(repo_id, &commits);
        
        assert_eq!(collaborations.len(), 1);
        assert_eq!(collaborations[0].authors.len(), 2);
        assert_eq!(collaborations[0].shared_files.len(), 2);
        assert!(collaborations[0].collaboration_strength > 0.5);
    }
}