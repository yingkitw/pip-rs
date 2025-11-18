/// Package cache for storing downloaded packages and metadata
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PackageCache {
    cache_dir: PathBuf,
    max_age_secs: u64,
}

impl PackageCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            cache_dir,
            max_age_secs: 86400, // 24 hours
        })
    }

    pub fn with_max_age(mut self, secs: u64) -> Self {
        self.max_age_secs = secs;
        self
    }

    pub fn get_cache_path(&self, package_name: &str, version: &str) -> PathBuf {
        self.cache_dir.join(format!("{}-{}", package_name, version))
    }

    pub fn is_cached(&self, package_name: &str, version: &str) -> bool {
        let path = self.get_cache_path(package_name, version);
        if !path.exists() {
            return false;
        }

        // Check if cache is still valid
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    return elapsed.as_secs() < self.max_age_secs;
                }
            }
        }

        false
    }

    pub fn store(&self, package_name: &str, version: &str, data: &[u8]) -> Result<PathBuf> {
        let path = self.get_cache_path(package_name, version);
        fs::write(&path, data)?;
        Ok(path)
    }

    pub fn retrieve(&self, package_name: &str, version: &str) -> Result<Vec<u8>> {
        let path = self.get_cache_path(package_name, version);
        Ok(fs::read(&path)?)
    }

    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn cleanup_old(&self) -> Result<u32> {
        let mut removed = 0;
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > self.max_age_secs {
                            fs::remove_file(&path)?;
                            removed += 1;
                        }
                    }
                }
            }
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache = PackageCache::new(temp_dir.path().to_path_buf())?;

        // Test storing
        let data = b"test data";
        cache.store("requests", "2.28.0", data)?;

        // Test retrieval
        let retrieved = cache.retrieve("requests", "2.28.0")?;
        assert_eq!(retrieved, data);

        // Test cache check
        assert!(cache.is_cached("requests", "2.28.0"));
        assert!(!cache.is_cached("requests", "2.29.0"));

        Ok(())
    }
}
