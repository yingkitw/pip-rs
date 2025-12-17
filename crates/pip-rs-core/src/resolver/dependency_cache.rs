/// Dependency caching for resolver optimization
/// 
/// This module implements caching for dependency iteration to avoid
/// redundant parsing and evaluation of package dependencies.

use std::collections::HashMap;
use crate::models::{Package, Requirement};
use anyhow::Result;

/// Cached dependency information
#[derive(Clone, Debug)]
pub struct CachedDependencies {
    pub package_name: String,
    pub version: String,
    pub dependencies: Vec<Requirement>,
    pub extras: Vec<String>,
}

/// Dependency cache for resolver
pub struct DependencyCache {
    cache: HashMap<String, CachedDependencies>,
    hits: u32,
    misses: u32,
}

impl DependencyCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Get cache key for a package
    fn cache_key(package_name: &str, version: &str) -> String {
        format!("{}=={}", package_name.to_lowercase(), version)
    }

    /// Get cached dependencies
    pub fn get(&mut self, package_name: &str, version: &str) -> Option<CachedDependencies> {
        let key = Self::cache_key(package_name, version);
        if let Some(deps) = self.cache.get(&key) {
            self.hits += 1;
            tracing::debug!(
                "Dependency cache hit for {}: {} hits total",
                key,
                self.hits
            );
            return Some(deps.clone());
        }
        self.misses += 1;
        None
    }

    /// Set cached dependencies
    pub fn set(
        &mut self,
        package_name: String,
        version: String,
        dependencies: Vec<Requirement>,
        extras: Vec<String>,
    ) {
        let key = Self::cache_key(&package_name, &version);
        let cached = CachedDependencies {
            package_name: package_name.clone(),
            version: version.clone(),
            dependencies,
            extras,
        };
        self.cache.insert(key.clone(), cached);
        tracing::debug!(
            "Cached dependencies for {}: {} misses total",
            key,
            self.misses
        );
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits: self.hits,
            misses: self.misses,
            total,
            hit_rate,
            size: self.cache.len(),
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
        tracing::debug!("Dependency cache cleared");
    }

    /// Print cache statistics
    pub fn print_stats(&self) {
        let stats = self.stats();
        println!("\n=== Dependency Cache Statistics ===");
        println!("Hits: {}", stats.hits);
        println!("Misses: {}", stats.misses);
        println!("Total: {}", stats.total);
        println!("Hit Rate: {:.1}%", stats.hit_rate);
        println!("Cached Packages: {}", stats.size);
    }
}

impl Default for DependencyCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u32,
    pub misses: u32,
    pub total: u32,
    pub hit_rate: f64,
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_cache_new() {
        let cache = DependencyCache::new();
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn test_dependency_cache_set_and_get() {
        let mut cache = DependencyCache::new();
        let deps = vec![];
        let extras = vec![];

        cache.set("requests".to_string(), "2.28.0".to_string(), deps.clone(), extras);

        let result = cache.get("requests", "2.28.0");
        assert!(result.is_some());
        assert_eq!(result.unwrap().package_name, "requests");
    }

    #[test]
    fn test_dependency_cache_miss() {
        let mut cache = DependencyCache::new();
        let result = cache.get("nonexistent", "1.0.0");
        assert!(result.is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_dependency_cache_hit_rate() {
        let mut cache = DependencyCache::new();
        let deps = vec![];
        let extras = vec![];

        cache.set("pkg1".to_string(), "1.0.0".to_string(), deps.clone(), extras.clone());
        cache.set("pkg2".to_string(), "2.0.0".to_string(), deps.clone(), extras);

        // 2 hits
        cache.get("pkg1", "1.0.0");
        cache.get("pkg2", "2.0.0");

        // 1 miss
        cache.get("pkg3", "3.0.0");

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total, 3);
        assert!((stats.hit_rate - 66.666).abs() < 1.0);
    }

    #[test]
    fn test_dependency_cache_clear() {
        let mut cache = DependencyCache::new();
        let deps = vec![];
        let extras = vec![];

        cache.set("pkg".to_string(), "1.0.0".to_string(), deps, extras);
        assert_eq!(cache.stats().size, 1);

        cache.clear();
        assert_eq!(cache.stats().size, 0);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_key_normalization() {
        let key1 = DependencyCache::cache_key("Requests", "2.28.0");
        let key2 = DependencyCache::cache_key("requests", "2.28.0");
        assert_eq!(key1, key2);
    }
}
