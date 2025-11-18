/// HTTP client for package operations
use anyhow::Result;
use reqwest::Client;

pub struct PackageClient {
    client: Client,
    base_url: String,
}

impl PackageClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://pypi.org/pypi".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub async fn get_package_info(&self, package_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}/json", self.base_url, package_name);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub async fn download_package(&self, url: &str) -> Result<bytes::Bytes> {
        let response = self.client.get(url).send().await?;
        let data = response.bytes().await?;
        Ok(data)
    }
}

impl Default for PackageClient {
    fn default() -> Self {
        Self::new()
    }
}
