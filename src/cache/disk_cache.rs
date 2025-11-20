use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};

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

    fn get_cache_path(&self, key: &str) -> PathBuf {
        let mut path = self.cache_dir.clone();
        path.push(key);
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
            std::fs::remove_file(&path)?;
            return Ok(None);
        }

        Ok(Some(std::fs::read(&path)?))
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.get_cache_path(key);
        std::fs::write(&path, value)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}