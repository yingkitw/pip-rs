use crate::models::Package;
use crate::cache::disk_cache::DiskCache;
use anyhow::Result;
use std::time::Duration;

pub struct PackageCache {
    disk_cache: DiskCache,
}

impl PackageCache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir().unwrap().join("pip-rs").join("packages");
        Self::new_custom(cache_dir)
    }

    pub fn new_custom(cache_dir: std::path::PathBuf) -> Result<Self> {
        let disk_cache = DiskCache::new(&cache_dir, Duration::from_secs(60 * 60 * 24))?; // 24 hour TTL
        Ok(Self { disk_cache })
    }

    pub fn get(&self, package_name: &str, version: &str) -> Result<Option<Package>> {
        let key = format!("{}-{}", package_name, version);
        if let Some(data) = self.disk_cache.get(&key)? {
            let package: Package = serde_json::from_slice(&data)?;
            return Ok(Some(package));
        }
        Ok(None)
    }

    pub fn set(&self, package: &Package) -> Result<()> {
        let key = format!("{}-{}", package.name, package.version);
        let data = serde_json::to_vec(package)?;
        self.disk_cache.set(&key, &data)?;
        Ok(())
    }
}

impl Default for PackageCache {
    fn default() -> Self {
        Self::new().expect("Failed to create package cache")
    }
}