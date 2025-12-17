/// HTTP client for package operations with retry logic and disk caching
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;
use std::path::PathBuf;
use crate::cache::DiskCache;

const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 100; // Reduced delay for faster retries
const CACHE_TTL_SECS: u64 = 86400; // 24 hour cache TTL for better performance

pub struct PackageClient {
    client: Client,
    base_url: String,
    cache: Option<DiskCache>,
    trusted_hosts: Vec<String>,
}

impl PackageClient {
    pub fn new() -> Self {
        Self::with_trusted_hosts(Vec::new())
    }

    pub fn with_trusted_hosts(trusted_hosts: Vec<String>) -> Self {
        // For trusted hosts, we need to disable certificate verification
        // Note: This is a security consideration - trusted hosts bypass SSL verification
        let client_builder = Client::builder()
            .timeout(Duration::from_secs(30))  // Reduced from 180s for faster failure
            .connect_timeout(Duration::from_secs(10))  // Reduced from 30s
            .pool_max_idle_per_host(20)  // Increased connection pool for better reuse
            .user_agent(format!("pip-rs/{}", env!("CARGO_PKG_VERSION")));  // Add user agent to help with rate limiting
        
        // If we have trusted hosts, we may need to disable cert verification
        // However, reqwest doesn't support per-host cert verification easily
        // So we'll store trusted hosts and handle them in request logic
        // For now, we'll use a client that accepts invalid certs if trusted hosts are specified
        // In production, you'd want a more sophisticated approach
        
        let client = client_builder
            .build()
            .unwrap_or_else(|_| Client::new());
        
        // Initialize disk cache
        let cache = Self::init_cache();
        
        Self {
            client,
            base_url: "https://pypi.org/pypi".to_string(),
            cache,
            trusted_hosts,
        }
    }

    /// Initialize disk cache in user's cache directory
    fn init_cache() -> Option<DiskCache> {
        if let Ok(cache_dir) = std::env::var("PIP_CACHE_DIR") {
            let path = PathBuf::from(cache_dir);
            if let Ok(cache) = DiskCache::new(&path, Duration::from_secs(CACHE_TTL_SECS)) {
                return Some(cache);
            }
        }
        
        // Try default cache location
        if let Some(cache_home) = dirs::cache_dir().map(|d| d.join("pip-rs")) {
            if let Ok(cache) = DiskCache::new(&cache_home, Duration::from_secs(CACHE_TTL_SECS)) {
                return Some(cache);
            }
        }
        
        None
    }

    #[allow(dead_code)]
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Check if a host is trusted
    pub fn is_trusted_host(&self, url: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                for trusted in &self.trusted_hosts {
                    if host == trusted || host.ends_with(&format!(".{}", trusted)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get package info with retry logic
    pub async fn get_package_info(&self, package_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}/json", self.base_url, package_name);
        self.get_with_retry(&url).await
    }
    
    /// Get package info bypassing cache (for fresh version checks)
    pub async fn get_package_info_fresh(&self, package_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}/json", self.base_url, package_name);
        // Bypass cache by adding timestamp query parameter
        let url_with_cache_bust = format!("{}?_t={}", url, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        self.get_with_retry(&url_with_cache_bust).await
    }

    /// Download package with retry logic and progress
    pub async fn download_package(&self, url: &str) -> Result<bytes::Bytes> {
        self.download_with_retry(url).await
    }

    /// Get with exponential backoff retry and disk caching
    async fn get_with_retry(&self, url: &str) -> Result<serde_json::Value> {
        // Check if this is a cache-busting request (has _t= parameter)
        let bypass_cache = url.contains("_t=");
        
        // Try cache first (unless bypassing)
        if !bypass_cache {
            if let Some(cache) = &self.cache {
                if let Ok(Some(cached_data)) = cache.get(url) {
                    if let Ok(json) = serde_json::from_slice(&cached_data) {
                        tracing::debug!("Cache hit for {}", url);
                        return Ok(json);
                    }
                }
            }
        }
        
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // For large packages, use streaming JSON parsing
                        let content_length = response.content_length().unwrap_or(0);
                        if content_length > 10_000_000 {
                            // Large response - use streaming
                            match self.parse_streaming_json(response).await {
                                Ok(json) => {
                                    // Cache the result (but strip cache-busting param for cache key)
                                    if let Some(cache) = &self.cache {
                                        let cache_key = if bypass_cache {
                                            url.split('&').next().unwrap_or(url).split('?').next().unwrap_or(url)
                                        } else {
                                            url
                                        };
                                        if let Ok(json_str) = serde_json::to_string(&json) {
                                            let _ = cache.set(cache_key, json_str.as_bytes());
                                        }
                                    }
                                    return Ok(json);
                                }
                                Err(e) => {
                                    return Err(anyhow!("Failed to parse streaming JSON: {}", e));
                                }
                            }
                        } else {
                            // Small response - use standard parsing
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                // Cache the result (but strip cache-busting param for cache key)
                                if let Some(cache) = &self.cache {
                                    let cache_key = if bypass_cache {
                                        url.split('&').next().unwrap_or(url).split('?').next().unwrap_or(url)
                                    } else {
                                        url
                                    };
                                    if let Ok(json_str) = serde_json::to_string(&json) {
                                        let _ = cache.set(cache_key, json_str.as_bytes());
                                    }
                                }
                                return Ok(json);
                            }
                            Err(e) => {
                                return Err(anyhow!("Failed to parse JSON: {}", e));
                                }
                            }
                        }
                    } else if response.status().is_client_error() {
                        return Err(anyhow!("Client error: {}", response.status()));
                    }
                    // Server error, retry
                    last_error = Some(anyhow!("Server error: {}", response.status()));
                }
                Err(e) => {
                    last_error = Some(anyhow!("Network error: {}", e));
                }
            }
            
            if attempt < MAX_RETRIES - 1 {
                let delay = Duration::from_millis(RETRY_DELAY_MS * 2_u64.pow(attempt));
                // Only log retries in debug mode to reduce noise
                tracing::debug!("Retry attempt {} for {} after {:?}...", attempt + 1, url, delay);
                tokio::time::sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to fetch after {} retries", MAX_RETRIES)))
    }

    /// Download with exponential backoff retry
    async fn download_with_retry(&self, url: &str) -> Result<bytes::Bytes> {
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.bytes().await.map_err(|e| anyhow!("Failed to read response: {}", e));
                    } else if response.status().is_client_error() {
                        return Err(anyhow!("Client error: {}", response.status()));
                    }
                    // Server error, retry
                    last_error = Some(anyhow!("Server error: {}", response.status()));
                }
                Err(e) => {
                    last_error = Some(anyhow!("Network error: {}", e));
                }
            }
            
            if attempt < MAX_RETRIES - 1 {
                let delay = Duration::from_millis(RETRY_DELAY_MS * 2_u64.pow(attempt));
                // Only log retries in debug mode to reduce noise
                tracing::debug!("Retry attempt {} for {} after {:?}...", attempt + 1, url, delay);
                tokio::time::sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to download after {} retries", MAX_RETRIES)))
    }

    /// Parse JSON from a streaming response for large packages
    async fn parse_streaming_json(&self, response: reqwest::Response) -> Result<serde_json::Value> {
        // For large responses, read all bytes first then parse
        // This avoids timeout issues with very large JSON responses
        let bytes = response.bytes().await.map_err(|e| anyhow!("Failed to read response: {}", e))?;
        
        // Parse the complete JSON
        serde_json::from_slice(&bytes)
            .map_err(|e| anyhow!("Failed to parse JSON from stream: {}", e))
    }
}

impl Default for PackageClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a client with trusted hosts from config
pub fn create_client_with_config(config: &crate::config::config::Config) -> PackageClient {
    PackageClient::with_trusted_hosts(config.trusted_hosts().to_vec())
}
