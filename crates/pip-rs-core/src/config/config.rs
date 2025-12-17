/// Configuration file parsing and management
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

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
        let mut config = Self {
            index_url: "https://pypi.org/simple/".to_string(),
            extra_index_urls: Vec::new(),
            timeout: 15,
            retries: 3,
            cache_dir: PathBuf::from(".pip-cache"),
            user_agent: "pip-rs/1.0.0".to_string(),
            trusted_hosts: Vec::new(),
        };
        
        // Load from environment variables
        config.load_from_env();
        
        // Load from pip.conf files
        config.load_from_standard_locations();
        
        config
    }

    /// Load configuration from environment variables
    fn load_from_env(&mut self) {
        // PIP_INDEX_URL
        if let Ok(url) = std::env::var("PIP_INDEX_URL") {
            self.index_url = url;
        }
        
        // PIP_EXTRA_INDEX_URL (comma-separated)
        if let Ok(urls) = std::env::var("PIP_EXTRA_INDEX_URL") {
            for url in urls.split(',') {
                let url = url.trim().to_string();
                if !url.is_empty() {
                    self.extra_index_urls.push(url);
                }
            }
        }
        
        // PIP_TRUSTED_HOST (comma-separated)
        if let Ok(hosts) = std::env::var("PIP_TRUSTED_HOST") {
            for host in hosts.split(',') {
                let host = host.trim().to_string();
                if !host.is_empty() {
                    self.trusted_hosts.push(host);
                }
            }
        }
        
        // PIP_CACHE_DIR
        if let Ok(cache_dir) = std::env::var("PIP_CACHE_DIR") {
            self.cache_dir = PathBuf::from(cache_dir);
        }
    }

    /// Load configuration from standard pip.conf locations
    fn load_from_standard_locations(&mut self) {
        use std::path::PathBuf;
        
        // Try user config first: ~/.pip/pip.conf (Unix) or %APPDATA%\pip\pip.ini (Windows)
        let user_config = if cfg!(windows) {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                PathBuf::from(appdata).join("pip").join("pip.ini")
            } else {
                return;
            }
        } else {
            if let Some(home) = dirs::home_dir() {
                home.join(".pip").join("pip.conf")
            } else {
                return;
            }
        };
        
        if let Ok(config) = Self::load_from_file(&user_config) {
            self.merge(&config);
        }
        
        // Try site config: /etc/pip.conf (Unix) or C:\ProgramData\pip\pip.ini (Windows)
        let site_config = if cfg!(windows) {
            PathBuf::from("C:\\ProgramData\\pip\\pip.ini")
        } else {
            PathBuf::from("/etc/pip.conf")
        };
        
        if let Ok(config) = Self::load_from_file(&site_config) {
            self.merge(&config);
        }
    }

    /// Merge another config into this one
    fn merge(&mut self, other: &Self) {
        // Only merge if values are different from defaults
        if other.index_url != "https://pypi.org/simple/" {
            self.index_url = other.index_url.clone();
        }
        self.extra_index_urls.extend_from_slice(&other.extra_index_urls);
        if other.timeout != 15 {
            self.timeout = other.timeout;
        }
        if other.retries != 3 {
            self.retries = other.retries;
        }
        if other.cache_dir != PathBuf::from(".pip-cache") {
            self.cache_dir = other.cache_dir.clone();
        }
        self.trusted_hosts.extend_from_slice(&other.trusted_hosts);
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
            return Ok(Self {
                index_url: "https://pypi.org/simple/".to_string(),
                extra_index_urls: Vec::new(),
                timeout: 15,
                retries: 3,
                cache_dir: PathBuf::from(".pip-cache"),
                user_agent: "pip-rs/1.0.0".to_string(),
                trusted_hosts: Vec::new(),
            });
        }

        let content = fs::read_to_string(path)?;
        let mut config = Self {
            index_url: "https://pypi.org/simple/".to_string(),
            extra_index_urls: Vec::new(),
            timeout: 15,
            retries: 3,
            cache_dir: PathBuf::from(".pip-cache"),
            user_agent: "pip-rs/1.0.0".to_string(),
            trusted_hosts: Vec::new(),
        };

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse section headers [section]
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_lowercase();
                continue;
            }

            // Parse key = value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                // Only process [global] section or if no section specified
                if current_section.is_empty() || current_section == "global" {
                    match key.as_str() {
                        "index-url" | "index_url" => config.index_url = value.to_string(),
                        "extra-index-url" | "extra_index_url" => {
                            config.extra_index_urls.push(value.to_string());
                        }
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
                        "cache-dir" | "cache_dir" => config.cache_dir = PathBuf::from(value),
                        "trusted-host" | "trusted_host" => {
                            config.trusted_hosts.push(value.to_string());
                        }
                        "find-links" | "find_links" => {
                            // Store as extra index URL for now
                            config.extra_index_urls.push(value.to_string());
                        }
                    _ => {}
                    }
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
