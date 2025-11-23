/// Editable mode caching for metadata optimization
/// 
/// This module implements caching for editable package information
/// to avoid redundant computation of editable_project_location.

use std::collections::HashMap;
use std::path::PathBuf;

/// Cached editable package information
#[derive(Clone, Debug)]
pub struct EditableInfo {
    pub package_name: String,
    pub version: String,
    pub project_location: Option<PathBuf>,
    pub is_editable: bool,
}

/// Cache for editable package information
pub struct EditableCache {
    cache: HashMap<String, EditableInfo>,
    hits: u32,
    misses: u32,
}

impl EditableCache {
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

    /// Get cached editable info
    pub fn get(&mut self, package_name: &str, version: &str) -> Option<EditableInfo> {
        let key = Self::cache_key(package_name, version);
        if let Some(info) = self.cache.get(&key) {
            self.hits += 1;
            tracing::debug!(
                "Editable cache hit for {}: {} hits total",
                key,
                self.hits
            );
            return Some(info.clone());
        }
        self.misses += 1;
        None
    }

    /// Set cached editable info
    pub fn set(
        &mut self,
        package_name: String,
        version: String,
        project_location: Option<PathBuf>,
        is_editable: bool,
    ) {
        let key = Self::cache_key(&package_name, &version);
        let info = EditableInfo {
            package_name: package_name.clone(),
            version: version.clone(),
            project_location,
            is_editable,
        };
        self.cache.insert(key.clone(), info);
        tracing::debug!(
            "Cached editable info for {}: {} misses total",
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
        tracing::debug!("Editable cache cleared");
    }

    /// Print cache statistics
    pub fn print_stats(&self) {
        let stats = self.stats();
        println!("\n=== Editable Cache Statistics ===");
        println!("Hits: {}", stats.hits);
        println!("Misses: {}", stats.misses);
        println!("Total: {}", stats.total);
        println!("Hit Rate: {:.1}%", stats.hit_rate);
        println!("Cached Packages: {}", stats.size);
    }
}

impl Default for EditableCache {
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
    fn test_editable_cache_new() {
        let cache = EditableCache::new();
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn test_editable_cache_set_and_get() {
        let mut cache = EditableCache::new();
        let location = Some(PathBuf::from("/path/to/package"));

        cache.set("requests".to_string(), "2.28.0".to_string(), location.clone(), true);

        let result = cache.get("requests", "2.28.0");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.package_name, "requests");
        assert!(info.is_editable);
        assert_eq!(info.project_location, location);
    }

    #[test]
    fn test_editable_cache_miss() {
        let mut cache = EditableCache::new();
        let result = cache.get("nonexistent", "1.0.0");
        assert!(result.is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_editable_cache_hit_rate() {
        let mut cache = EditableCache::new();

        cache.set("pkg1".to_string(), "1.0.0".to_string(), None, false);
        cache.set("pkg2".to_string(), "2.0.0".to_string(), None, false);

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
    fn test_editable_cache_clear() {
        let mut cache = EditableCache::new();

        cache.set("pkg".to_string(), "1.0.0".to_string(), None, false);
        assert_eq!(cache.stats().size, 1);

        cache.clear();
        assert_eq!(cache.stats().size, 0);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_key_normalization() {
        let key1 = EditableCache::cache_key("Requests", "2.28.0");
        let key2 = EditableCache::cache_key("requests", "2.28.0");
        assert_eq!(key1, key2);
    }
}
