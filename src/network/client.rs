/// HTTP client for package operations with retry logic
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;

const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 200;

pub struct PackageClient {
    client: Client,
    base_url: String,
}

impl PackageClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)  // Connection pooling
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            base_url: "https://pypi.org/pypi".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Get package info with retry logic
    pub async fn get_package_info(&self, package_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}/json", self.base_url, package_name);
        self.get_with_retry(&url).await
    }

    /// Download package with retry logic and progress
    pub async fn download_package(&self, url: &str) -> Result<bytes::Bytes> {
        self.download_with_retry(url).await
    }

    /// Get with exponential backoff retry
    async fn get_with_retry(&self, url: &str) -> Result<serde_json::Value> {
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.json().await.map_err(|e| anyhow!("Failed to parse JSON: {}", e));
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
                eprintln!("Retry attempt {} after {:?}...", attempt + 1, delay);
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
                eprintln!("Retry attempt {} after {:?}...", attempt + 1, delay);
                tokio::time::sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to download after {} retries", MAX_RETRIES)))
    }
}

impl Default for PackageClient {
    fn default() -> Self {
        Self::new()
    }
}
