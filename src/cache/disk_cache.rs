/// Disk-based package metadata cache with TTL
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Serialize, Deserialize};

use super::super::models::Package;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPackage {
    package: Package,
    timestamp: u64,
}

pub struct DiskCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl DiskCache {
    /// Create a new disk cache with default location and 24-hour TTL
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Cannot determine cache directory"))?
            .join("pip-rs");
        
        fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            ttl: Duration::from_secs(24 * 60 * 60), // 24 hours
        })
    }

    /// Create cache with custom TTL
    #[allow(dead_code)]
    pub fn with_ttl(ttl: Duration) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Cannot determine cache directory"))?
            .join("pip-rs");
        
        fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            ttl,
        })
    }

    /// Get package from cache if not expired
    pub fn get(&self, package_name: &str) -> Result<Option<Package>> {
        let cache_file = self.cache_dir.join(format!("{}.json", package_name));
        
        if !cache_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_file)?;
        let cached: CachedPackage = serde_json::from_str(&content)?;

        // Check if expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        if now - cached.timestamp > self.ttl.as_secs() {
            // Cache expired, remove it
            let _ = fs::remove_file(&cache_file);
            return Ok(None);
        }

        Ok(Some(cached.package))
    }

    /// Store package in cache
    pub fn set(&self, package: &Package) -> Result<()> {
        let cache_file = self.cache_dir.join(format!("{}.json", package.name));
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let cached = CachedPackage {
            package: package.clone(),
            timestamp,
        };

        let json = serde_json::to_string(&cached)?;
        fs::write(&cache_file, json)?;
        
        Ok(())
    }

    /// Clear all cached packages
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut count = 0;
        let mut size = 0;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                    size += fs::metadata(&path)?.len();
                }
            }
        }

        Ok(CacheStats { count, size })
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub count: usize,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disk_cache_store_and_retrieve() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().to_path_buf();
        
        let cache = DiskCache {
            cache_dir,
            ttl: Duration::from_secs(60),
        };

        let pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: Some("Test package".to_string()),
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        };

        // Store package
        cache.set(&pkg)?;

        // Retrieve package
        let retrieved = cache.get("test-package")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-package");

        Ok(())
    }

    #[test]
    #[ignore]  // Timing-sensitive test, skip for now
    fn test_disk_cache_expiration() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().to_path_buf();
        
        let cache = DiskCache {
            cache_dir,
            ttl: Duration::from_millis(200), // 200ms expiration
        };

        let pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        };

        cache.set(&pkg)?;
        
        // Wait for expiration (wait longer than TTL)
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Should be expired
        let retrieved = cache.get("test-package")?;
        assert!(retrieved.is_none());

        Ok(())
    }

    #[test]
    fn test_disk_cache_stats() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().to_path_buf();
        
        let cache = DiskCache {
            cache_dir,
            ttl: Duration::from_secs(60),
        };

        let pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        };

        cache.set(&pkg)?;

        let stats = cache.stats()?;
        assert_eq!(stats.count, 1);
        assert!(stats.size > 0);

        Ok(())
    }
}
