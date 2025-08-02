// Copyright 2025 Cowboy AI, LLC.

//! Caching layer for Git domain operations
//!
//! Provides caching for expensive operations like repository analysis
//! and commit analysis to improve performance.

use crate::{value_objects::CommitHash, RepositoryId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Cache for Git domain operations
pub struct GitCache {
    /// Cached repository analysis results
    repository_analysis: Arc<RwLock<HashMap<RepositoryId, CacheEntry<RepositoryAnalysis>>>>,
    /// Cached commit analysis results
    commit_analysis: Arc<RwLock<HashMap<(RepositoryId, CommitHash), CacheEntry<CommitAnalysis>>>>,
    /// Default time-to-live for cache entries
    default_ttl: Duration,
}

/// Cached repository analysis
#[derive(Clone)]
pub struct RepositoryAnalysis {
    /// Number of branches
    pub branch_count: usize,
    /// Number of commits
    pub commit_count: usize,
    /// Repository size in bytes
    pub size_bytes: u64,
}

/// Cached commit analysis
#[derive(Clone)]
pub struct CommitAnalysis {
    /// Number of files changed
    pub files_changed: usize,
    /// Lines added
    pub lines_added: usize,
    /// Lines deleted
    pub lines_deleted: usize,
}

impl GitCache {
    /// Create a new cache with default TTL of 5 minutes
    #[must_use]
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    /// Create a new cache with custom TTL
    #[must_use]
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            repository_analysis: Arc::new(RwLock::new(HashMap::new())),
            commit_analysis: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: ttl,
        }
    }

    /// Get cached repository analysis
    #[must_use]
    pub fn get_repository_analysis(&self, repo_id: &RepositoryId) -> Option<RepositoryAnalysis> {
        let cache = self.repository_analysis.read().ok()?;

        cache
            .get(repo_id)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    /// Cache repository analysis
    pub fn cache_repository_analysis(&self, repo_id: RepositoryId, analysis: RepositoryAnalysis) {
        if let Ok(mut cache) = self.repository_analysis.write() {
            cache.insert(repo_id, CacheEntry::new(analysis, self.default_ttl));
        }
    }

    /// Get cached commit analysis
    #[must_use]
    pub fn get_commit_analysis(
        &self,
        repo_id: &RepositoryId,
        commit_hash: &CommitHash,
    ) -> Option<CommitAnalysis> {
        let key = (*repo_id, commit_hash.clone());
        let cache = self.commit_analysis.read().ok()?;

        cache
            .get(&key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    /// Cache commit analysis
    pub fn cache_commit_analysis(
        &self,
        repo_id: RepositoryId,
        commit_hash: CommitHash,
        analysis: CommitAnalysis,
    ) {
        let key = (repo_id, commit_hash);
        if let Ok(mut cache) = self.commit_analysis.write() {
            cache.insert(key, CacheEntry::new(analysis, self.default_ttl));
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        if let Ok(mut cache) = self.repository_analysis.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.commit_analysis.write() {
            cache.clear();
        }
    }

    /// Clear expired entries
    pub fn evict_expired(&self) {
        if let Ok(mut cache) = self.repository_analysis.write() {
            cache.retain(|_, entry| !entry.is_expired());
        }
        if let Ok(mut cache) = self.commit_analysis.write() {
            cache.retain(|_, entry| !entry.is_expired());
        }
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        let repository_analysis_count = self
            .repository_analysis
            .read()
            .map(|c| c.len())
            .unwrap_or(0);
        let commit_analysis_count = self.commit_analysis.read().map(|c| c.len()).unwrap_or(0);

        CacheStats {
            repository_analysis_entries: repository_analysis_count,
            commit_analysis_entries: commit_analysis_count,
        }
    }
}

impl Default for GitCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached repository analysis entries
    pub repository_analysis_entries: usize,
    /// Number of cached commit analysis entries
    pub commit_analysis_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_expiration() {
        let cache = GitCache::with_ttl(Duration::from_millis(100));
        let repo_id = RepositoryId::new();

        // Cache repository analysis
        let analysis = RepositoryAnalysis {
            branch_count: 5,
            commit_count: 100,
            size_bytes: 1024 * 1024,
        };
        cache.cache_repository_analysis(repo_id, analysis.clone());

        // Should be retrievable immediately
        assert!(cache.get_repository_analysis(&repo_id).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert!(cache.get_repository_analysis(&repo_id).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = GitCache::new();
        let repo_id = RepositoryId::new();
        let commit_hash = CommitHash::new("abcdef1234567890").unwrap();

        // Initially empty
        let stats = cache.stats();
        assert_eq!(stats.repository_analysis_entries, 0);
        assert_eq!(stats.commit_analysis_entries, 0);

        // Add repository analysis
        cache.cache_repository_analysis(
            repo_id,
            RepositoryAnalysis {
                branch_count: 3,
                commit_count: 50,
                size_bytes: 500_000,
            },
        );

        // Add commit analysis
        cache.cache_commit_analysis(
            repo_id,
            commit_hash,
            CommitAnalysis {
                files_changed: 10,
                lines_added: 100,
                lines_deleted: 50,
            },
        );

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.repository_analysis_entries, 1);
        assert_eq!(stats.commit_analysis_entries, 1);
    }
}
