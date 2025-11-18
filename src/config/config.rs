/// Configuration file parsing and management
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    index_url: String,
    extra_index_urls: Vec<String>,
    timeout: u64,
    retries: u32,
    cache_dir: PathBuf,
    user_agent: String,
    trusted_hosts: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            index_url: "https://pypi.org/simple/".to_string(),
            extra_index_urls: Vec::new(),
            timeout: 15,
            retries: 3,
            cache_dir: PathBuf::from(".pip-cache"),
            user_agent: "pip-rs/0.1.0".to_string(),
            trusted_hosts: Vec::new(),
        }
    }

    pub fn index_url(&self) -> &str {
        &self.index_url
    }

    pub fn set_index_url(&mut self, url: String) {
        self.index_url = url;
    }

    pub fn extra_index_urls(&self) -> &[String] {
        &self.extra_index_urls
    }

    pub fn add_extra_index_url(&mut self, url: String) {
        self.extra_index_urls.push(url);
    }

    pub fn timeout(&self) -> u64 {
        self.timeout
    }

    pub fn set_timeout(&mut self, secs: u64) {
        self.timeout = secs;
    }

    pub fn retries(&self) -> u32 {
        self.retries
    }

    pub fn set_retries(&mut self, count: u32) {
        self.retries = count;
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn set_cache_dir(&mut self, path: PathBuf) {
        self.cache_dir = path;
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn trusted_hosts(&self) -> &[String] {
        &self.trusted_hosts
    }

    pub fn add_trusted_host(&mut self, host: String) {
        self.trusted_hosts.push(host);
    }

    /// Load configuration from pip.ini or .pip/pip.conf
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)?;
        let mut config = Self::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key = value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "index-url" => config.index_url = value.to_string(),
                    "extra-index-url" => config.extra_index_urls.push(value.to_string()),
                    "timeout" => {
                        if let Ok(timeout) = value.parse::<u64>() {
                            config.timeout = timeout;
                        }
                    }
                    "retries" => {
                        if let Ok(retries) = value.parse::<u32>() {
                            config.retries = retries;
                        }
                    }
                    "cache-dir" => config.cache_dir = PathBuf::from(value),
                    "trusted-host" => config.trusted_hosts.push(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let mut content = String::from("[global]\n");
        
        content.push_str(&format!("index-url = {}\n", self.index_url));
        
        for url in &self.extra_index_urls {
            content.push_str(&format!("extra-index-url = {}\n", url));
        }
        
        content.push_str(&format!("timeout = {}\n", self.timeout));
        content.push_str(&format!("retries = {}\n", self.retries));
        content.push_str(&format!("cache-dir = {}\n", self.cache_dir.display()));
        
        for host in &self.trusted_hosts {
            content.push_str(&format!("trusted-host = {}\n", host));
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert_eq!(config.index_url(), "https://pypi.org/simple/");
        assert_eq!(config.timeout(), 15);
        assert_eq!(config.retries(), 3);
    }

    #[test]
    fn test_config_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("pip.conf");

        let mut config = Config::new();
        config.set_timeout(30);
        config.add_extra_index_url("https://test.pypi.org/simple/".to_string());
        config.save_to_file(&config_path)?;

        let loaded = Config::load_from_file(&config_path)?;
        assert_eq!(loaded.timeout(), 30);
        assert_eq!(loaded.extra_index_urls().len(), 1);

        Ok(())
    }
}
