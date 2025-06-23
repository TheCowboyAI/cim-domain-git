//! Caching layer for Git domain operations
//!
//! Provides caching for expensive operations like commit graph traversal
//! and dependency analysis to improve performance.

use crate::{RepositoryId, value_objects::CommitHash};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use cim_domain_graph::GraphId;

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
    /// Cached commit graphs
    commit_graphs: Arc<RwLock<HashMap<(RepositoryId, Option<CommitHash>), CacheEntry<GraphId>>>>,
    /// Cached dependency graphs
    dependency_graphs: Arc<RwLock<HashMap<(RepositoryId, CommitHash), CacheEntry<DependencyInfo>>>>,
    /// Default time-to-live for cache entries
    default_ttl: Duration,
}

/// Cached dependency information
#[derive(Clone)]
pub struct DependencyInfo {
    /// Graph ID for the dependency graph
    pub graph_id: GraphId,
    /// Number of files analyzed
    pub file_count: usize,
    /// Number of dependencies found
    pub dependency_count: usize,
}

impl GitCache {
    /// Create a new cache with default TTL of 5 minutes
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    /// Create a new cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            commit_graphs: Arc::new(RwLock::new(HashMap::new())),
            dependency_graphs: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: ttl,
        }
    }

    /// Get cached commit graph
    pub fn get_commit_graph(
        &self,
        repo_id: &RepositoryId,
        start_commit: Option<&CommitHash>,
    ) -> Option<GraphId> {
        let key = (*repo_id, start_commit.cloned());
        let cache = self.commit_graphs.read().ok()?;
        
        cache.get(&key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value)
    }

    /// Cache commit graph
    pub fn cache_commit_graph(
        &self,
        repo_id: RepositoryId,
        start_commit: Option<CommitHash>,
        graph_id: GraphId,
    ) {
        let key = (repo_id, start_commit);
        if let Ok(mut cache) = self.commit_graphs.write() {
            cache.insert(key, CacheEntry::new(graph_id, self.default_ttl));
        }
    }

    /// Get cached dependency graph
    pub fn get_dependency_graph(
        &self,
        repo_id: &RepositoryId,
        commit_hash: &CommitHash,
    ) -> Option<DependencyInfo> {
        let key = (*repo_id, commit_hash.clone());
        let cache = self.dependency_graphs.read().ok()?;
        
        cache.get(&key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    /// Cache dependency graph
    pub fn cache_dependency_graph(
        &self,
        repo_id: RepositoryId,
        commit_hash: CommitHash,
        info: DependencyInfo,
    ) {
        let key = (repo_id, commit_hash);
        if let Ok(mut cache) = self.dependency_graphs.write() {
            cache.insert(key, CacheEntry::new(info, self.default_ttl));
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        if let Ok(mut cache) = self.commit_graphs.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.dependency_graphs.write() {
            cache.clear();
        }
    }

    /// Clear expired entries
    pub fn evict_expired(&self) {
        if let Ok(mut cache) = self.commit_graphs.write() {
            cache.retain(|_, entry| !entry.is_expired());
        }
        if let Ok(mut cache) = self.dependency_graphs.write() {
            cache.retain(|_, entry| !entry.is_expired());
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let commit_graph_count = self.commit_graphs.read()
            .map(|c| c.len())
            .unwrap_or(0);
        let dependency_graph_count = self.dependency_graphs.read()
            .map(|c| c.len())
            .unwrap_or(0);

        CacheStats {
            commit_graph_entries: commit_graph_count,
            dependency_graph_entries: dependency_graph_count,
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
    /// Number of cached commit graph entries
    pub commit_graph_entries: usize,
    /// Number of cached dependency graph entries
    pub dependency_graph_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_expiration() {
        let cache = GitCache::with_ttl(Duration::from_millis(100));
        let repo_id = RepositoryId::new();
        let graph_id = GraphId::new();

        // Cache a commit graph
        cache.cache_commit_graph(repo_id, None, graph_id);

        // Should be retrievable immediately
        assert_eq!(cache.get_commit_graph(&repo_id, None), Some(graph_id));

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert_eq!(cache.get_commit_graph(&repo_id, None), None);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = GitCache::with_ttl(Duration::from_millis(100));
        let repo_id = RepositoryId::new();
        let commit_hash = CommitHash::new("abc123def456789").unwrap();
        let info = DependencyInfo {
            graph_id: GraphId::new(),
            file_count: 10,
            dependency_count: 20,
        };

        // Cache dependency info
        cache.cache_dependency_graph(repo_id, commit_hash.clone(), info.clone());

        // Stats should show 1 entry
        let stats = cache.stats();
        assert_eq!(stats.dependency_graph_entries, 1);

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Evict expired entries
        cache.evict_expired();

        // Stats should show 0 entries
        let stats = cache.stats();
        assert_eq!(stats.dependency_graph_entries, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = GitCache::new();
        let repo_id = RepositoryId::new();
        let graph_id = GraphId::new();

        // Cache multiple entries
        cache.cache_commit_graph(repo_id, None, graph_id);
        cache.cache_commit_graph(repo_id, Some(CommitHash::new("abc123def456789").unwrap()), graph_id);

        let stats = cache.stats();
        assert_eq!(stats.commit_graph_entries, 2);

        // Clear cache
        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.commit_graph_entries, 0);
    }
} 