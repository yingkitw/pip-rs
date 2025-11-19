/// PyPI index management and support for multiple indexes
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// PyPI index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index name
    pub name: String,
    /// Index URL
    pub url: String,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether this is the default index
    pub default: bool,
    /// Optional authentication token
    pub token: Option<String>,
}

/// Index manager for handling multiple PyPI indexes
pub struct IndexManager {
    /// Primary index (default PyPI)
    primary: IndexConfig,
    /// Additional indexes
    secondary: Vec<IndexConfig>,
}

impl IndexManager {
    /// Create a new index manager with default PyPI
    pub fn new() -> Self {
        Self {
            primary: IndexConfig {
                name: "pypi".to_string(),
                url: "https://pypi.org/simple/".to_string(),
                priority: 0,
                default: true,
                token: None,
            },
            secondary: Vec::new(),
        }
    }

    /// Add a secondary index
    pub fn add_index(&mut self, mut index: IndexConfig) -> Result<()> {
        if index.url.is_empty() {
            return Err(anyhow!("Index URL cannot be empty"));
        }

        // Ensure URL ends with /
        if !index.url.ends_with('/') {
            index.url = format!("{}/", index.url);
        }

        self.secondary.push(index);
        self.secondary.sort_by_key(|idx| idx.priority);

        Ok(())
    }

    /// Get all indexes sorted by priority
    pub fn get_all_indexes(&self) -> Vec<IndexConfig> {
        let mut indexes = vec![self.primary.clone()];
        indexes.extend(self.secondary.clone());
        indexes.sort_by_key(|idx| idx.priority);
        indexes
    }

    /// Get primary index
    pub fn get_primary(&self) -> &IndexConfig {
        &self.primary
    }

    /// Set primary index
    pub fn set_primary(&mut self, index: IndexConfig) -> Result<()> {
        if index.url.is_empty() {
            return Err(anyhow!("Index URL cannot be empty"));
        }
        self.primary = index;
        Ok(())
    }

    /// Find an index by name
    pub fn find_index(&self, name: &str) -> Option<IndexConfig> {
        if self.primary.name == name {
            return Some(self.primary.clone());
        }
        self.secondary.iter().find(|idx| idx.name == name).cloned()
    }

    /// Get index URL for a package
    pub fn get_package_url(&self, index: &IndexConfig, package_name: &str) -> String {
        format!("{}{}/", index.url.trim_end_matches('/'), package_name)
    }

    /// Try to fetch from multiple indexes with fallback
    pub async fn fetch_with_fallback<F, T>(
        &self,
        package_name: &str,
        fetch_fn: F,
    ) -> Result<T>
    where
        F: Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>>>>,
    {
        let indexes = self.get_all_indexes();

        for (i, index) in indexes.iter().enumerate() {
            let url = self.get_package_url(index, package_name);
            
            match fetch_fn(&url).await {
                Ok(result) => {
                    tracing::info!(
                        "Successfully fetched {} from index: {}",
                        package_name,
                        index.name
                    );
                    return Ok(result);
                }
                Err(e) => {
                    if i < indexes.len() - 1 {
                        tracing::warn!(
                            "Failed to fetch {} from {}: {}. Trying next index...",
                            package_name,
                            index.name,
                            e
                        );
                    } else {
                        tracing::error!(
                            "Failed to fetch {} from all indexes. Last error: {}",
                            package_name,
                            e
                        );
                        return Err(anyhow!(
                            "Failed to fetch {} from any index: {}",
                            package_name,
                            e
                        ));
                    }
                }
            }
        }

        Err(anyhow!("No indexes available"))
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse index configuration from pip.conf format
pub fn parse_index_config(content: &str) -> Result<Vec<IndexConfig>> {
    let mut indexes = Vec::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse section headers [index-servers]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        // Parse key-value pairs
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if current_section == "index-servers" && key == "index-url" {
                indexes.push(IndexConfig {
                    name: "pypi".to_string(),
                    url: value.to_string(),
                    priority: 0,
                    default: true,
                    token: None,
                });
            } else if current_section == "index-servers" && key == "extra-index-url" {
                let priority = (indexes.len() as u32) + 1;
                indexes.push(IndexConfig {
                    name: format!("extra-{}", priority),
                    url: value.to_string(),
                    priority,
                    default: false,
                    token: None,
                });
            }
        }
    }

    Ok(indexes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_manager_creation() {
        let manager = IndexManager::new();
        assert_eq!(manager.get_primary().name, "pypi");
        assert!(manager.get_primary().default);
    }

    #[test]
    fn test_add_secondary_index() {
        let mut manager = IndexManager::new();
        let index = IndexConfig {
            name: "test".to_string(),
            url: "https://test.example.com/simple".to_string(),
            priority: 1,
            default: false,
            token: None,
        };
        manager.add_index(index).unwrap();
        assert_eq!(manager.secondary.len(), 1);
    }

    #[test]
    fn test_get_all_indexes() {
        let mut manager = IndexManager::new();
        let index = IndexConfig {
            name: "test".to_string(),
            url: "https://test.example.com/simple".to_string(),
            priority: 1,
            default: false,
            token: None,
        };
        manager.add_index(index).unwrap();
        let all = manager.get_all_indexes();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_find_index() {
        let mut manager = IndexManager::new();
        let index = IndexConfig {
            name: "test".to_string(),
            url: "https://test.example.com/simple".to_string(),
            priority: 1,
            default: false,
            token: None,
        };
        manager.add_index(index).unwrap();
        assert!(manager.find_index("test").is_some());
        assert!(manager.find_index("nonexistent").is_none());
    }

    #[test]
    fn test_get_package_url() {
        let manager = IndexManager::new();
        let index = manager.get_primary();
        let url = manager.get_package_url(index, "requests");
        assert!(url.contains("requests"));
        assert!(url.ends_with('/'));
    }

    #[test]
    fn test_parse_index_config() {
        let config = r#"
[index-servers]
index-url = https://pypi.org/simple/
extra-index-url = https://test.example.com/simple
"#;
        let indexes = parse_index_config(config).unwrap();
        assert_eq!(indexes.len(), 2);
    }

    #[test]
    fn test_url_normalization() {
        let mut manager = IndexManager::new();
        let index = IndexConfig {
            name: "test".to_string(),
            url: "https://test.example.com/simple".to_string(),
            priority: 1,
            default: false,
            token: None,
        };
        manager.add_index(index).unwrap();
        let added = manager.find_index("test").unwrap();
        assert!(added.url.ends_with('/'));
    }
}
