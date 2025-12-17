use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct DiskCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl DiskCache {
    pub fn new(cache_dir: &Path, ttl: Duration) -> Result<Self> {
        std::fs::create_dir_all(cache_dir)?;
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            ttl,
        })
    }

    /// Hash the key to create a safe filename
    fn hash_key(key: &str) -> String {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn get_cache_path(&self, key: &str) -> PathBuf {
        let hash = Self::hash_key(key);
        // Use first 2 chars as subdirectory for better filesystem performance
        let subdir = &hash[..2];
        let mut path = self.cache_dir.clone();
        path.push(subdir);
        let _ = std::fs::create_dir_all(&path);
        path.push(&hash);
        path
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_cache_path(key);
        if !path.exists() {
            return Ok(None);
        }

        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;
        if SystemTime::now().duration_since(modified)? > self.ttl {
            // Cache expired
            let _ = std::fs::remove_file(&path);
            return Ok(None);
        }

        Ok(Some(std::fs::read(&path)?))
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.get_cache_path(key);
        std::fs::write(&path, value)?;
        Ok(())
    }

    /// Get or fetch with async closure
    pub async fn get_or_fetch<F, Fut>(&self, key: &str, fetch: F) -> Result<Vec<u8>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<u8>>>,
    {
        if let Some(cached) = self.get(key)? {
            return Ok(cached);
        }
        
        let data = fetch().await?;
        self.set(key, &data)?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        let mut total_size = 0u64;
        let mut file_count = 0u64;
        
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total_size += meta.len();
                        file_count += 1;
                    } else if meta.is_dir() {
                        // Count files in subdirectory
                        if let Ok(subentries) = std::fs::read_dir(entry.path()) {
                            for subentry in subentries.flatten() {
                                if let Ok(submeta) = subentry.metadata() {
                                    if submeta.is_file() {
                                        total_size += submeta.len();
                                        file_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        CacheStats { total_size, file_count }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_size: u64,
    pub file_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_set_get() {
        let dir = tempdir().unwrap();
        let cache = DiskCache::new(dir.path(), Duration::from_secs(3600)).unwrap();
        
        cache.set("test_key", b"test_value").unwrap();
        let result = cache.get("test_key").unwrap();
        
        assert_eq!(result, Some(b"test_value".to_vec()));
    }

    #[test]
    fn test_cache_miss() {
        let dir = tempdir().unwrap();
        let cache = DiskCache::new(dir.path(), Duration::from_secs(3600)).unwrap();
        
        let result = cache.get("nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_hash_key() {
        let hash1 = DiskCache::hash_key("https://pypi.org/pypi/requests/json");
        let hash2 = DiskCache::hash_key("https://pypi.org/pypi/requests/json");
        let hash3 = DiskCache::hash_key("https://pypi.org/pypi/flask/json");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}