// Copyright 2025 Cowboy AI, LLC.

//! Collaboration-related events for graph analytics
//!
//! These events capture collaboration patterns that are valuable
//! for building contributor relationship graphs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    aggregate::RepositoryId,
    value_objects::{AuthorInfo, CommitHash, FilePath},
};

/// Detected when multiple authors work on the same files
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollaborationDetected {
    /// Repository where collaboration occurred
    pub repository_id: RepositoryId,
    
    /// Authors who collaborated
    pub authors: Vec<AuthorInfo>,
    
    /// Files they both modified
    pub shared_files: Vec<FilePath>,
    
    /// Time window of collaboration
    pub time_window_hours: u32,
    
    /// Strength of collaboration (0.0 to 1.0)
    /// Based on number of shared files and commit proximity
    pub collaboration_strength: f64,
    
    /// When this collaboration was detected
    pub timestamp: DateTime<Utc>,
}

/// Detected when commits reference each other (cherry-picks, reverts, etc)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitRelationshipDetected {
    /// Repository containing the commits
    pub repository_id: RepositoryId,
    
    /// Source commit
    pub source_commit: CommitHash,
    
    /// Target commit
    pub target_commit: CommitHash,
    
    /// Type of relationship
    pub relationship_type: CommitRelationshipType,
    
    /// When this relationship was detected
    pub timestamp: DateTime<Utc>,
}

/// Types of relationships between commits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitRelationshipType {
    /// Target cherry-picks source
    CherryPick,
    /// Target reverts source
    Revert,
    /// Target fixes source (based on commit message)
    Fixes,
    /// Target references source in message
    References,
    /// Target is a merge containing source
    MergeContains,
}

/// Detected when a code review pattern is identified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeReviewDetected {
    /// Repository where review occurred
    pub repository_id: RepositoryId,
    
    /// Author of the original commits
    pub author: AuthorInfo,
    
    /// Reviewer (identified by subsequent commits to same files)
    pub reviewer: AuthorInfo,
    
    /// Files reviewed
    pub reviewed_files: Vec<FilePath>,
    
    /// Review intensity (number of changes by reviewer)
    pub review_intensity: u32,
    
    /// When this review pattern was detected
    pub timestamp: DateTime<Utc>,
}

/// Detected ownership patterns in the codebase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeOwnershipCalculated {
    /// Repository analyzed
    pub repository_id: RepositoryId,
    
    /// File or directory path
    pub path: FilePath,
    
    /// Primary owner (most commits)
    pub primary_owner: AuthorInfo,
    
    /// Ownership percentage (0.0 to 1.0)
    pub ownership_percentage: f64,
    
    /// Other contributors and their percentages
    pub contributors: Vec<(AuthorInfo, f64)>,
    
    /// Total commits to this path
    pub total_commits: u32,
    
    /// When this calculation was performed
    pub timestamp: DateTime<Utc>,
}

/// Team formation detected based on collaboration patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamClusterDetected {
    /// Repository where team was detected
    pub repository_id: RepositoryId,
    
    /// Team members who frequently collaborate
    pub team_members: Vec<AuthorInfo>,
    
    /// Cohesion score (0.0 to 1.0)
    /// Based on how often team members work together vs with others
    pub cohesion_score: f64,
    
    /// Primary areas of focus for this team
    pub focus_areas: Vec<FilePath>,
    
    /// When this team pattern was detected
    pub timestamp: DateTime<Utc>,
}