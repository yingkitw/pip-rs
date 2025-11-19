/// Lock file support for reproducible installs
/// Generates and reads lock files in JSON format

use crate::models::Package;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Lock file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file version
    pub version: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Python version used
    pub python_version: String,
    /// Locked packages
    pub packages: HashMap<String, LockedPackage>,
}

/// A locked package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package name
    pub name: String,
    /// Exact version
    pub version: String,
    /// Package summary
    pub summary: Option<String>,
    /// Direct dependencies
    pub dependencies: Vec<String>,
    /// Installation hash (for verification)
    pub hash: Option<String>,
    /// Installation URL
    pub url: Option<String>,
}

impl LockFile {
    /// Create a new lock file from resolved packages
    pub fn from_packages(packages: Vec<Package>, python_version: String) -> Self {
        let mut locked_packages = HashMap::new();

        for pkg in packages {
            let key = format!("{}-{}", pkg.name, pkg.version);
            locked_packages.insert(
                key,
                LockedPackage {
                    name: pkg.name,
                    version: pkg.version,
                    summary: pkg.summary,
                    dependencies: pkg.requires_dist,
                    hash: None,
                    url: None,
                },
            );
        }

        Self {
            version: "1.0".to_string(),
            generated_at: chrono::Local::now().to_rfc3339(),
            python_version,
            packages: locked_packages,
        }
    }

    /// Save lock file to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load lock file from disk
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let lockfile = serde_json::from_str(&contents)?;
        Ok(lockfile)
    }

    /// Get all locked packages as Package objects
    pub fn to_packages(&self) -> Vec<Package> {
        self.packages
            .values()
            .map(|locked| Package {
                name: locked.name.clone(),
                version: locked.version.clone(),
                summary: locked.summary.clone(),
                home_page: None,
                author: None,
                license: None,
                requires_python: None,
                requires_dist: locked.dependencies.clone(),
                classifiers: vec![],
            })
            .collect()
    }

    /// Get a specific locked package
    pub fn get_package(&self, name: &str, version: &str) -> Option<LockedPackage> {
        let key = format!("{}-{}", name, version);
        self.packages.get(&key).cloned()
    }

    /// Check if package is locked
    pub fn has_package(&self, name: &str) -> bool {
        self.packages.values().any(|pkg| pkg.name == name)
    }

    /// Get all package names
    pub fn package_names(&self) -> Vec<String> {
        self.packages
            .values()
            .map(|pkg| pkg.name.clone())
            .collect()
    }

    /// Validate lock file integrity
    pub fn validate(&self) -> Result<()> {
        if self.version != "1.0" {
            return Err(anyhow::anyhow!("Unsupported lock file version: {}", self.version));
        }

        if self.packages.is_empty() {
            return Err(anyhow::anyhow!("Lock file contains no packages"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_creation() {
        let packages = vec![Package {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            summary: Some("HTTP library".to_string()),
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec!["urllib3>=1.21".to_string()],
            classifiers: vec![],
        }];

        let lockfile = LockFile::from_packages(packages, "3.11".to_string());
        assert_eq!(lockfile.version, "1.0");
        assert_eq!(lockfile.python_version, "3.11");
        assert_eq!(lockfile.packages.len(), 1);
    }

    #[test]
    fn test_lockfile_to_packages() {
        let packages = vec![Package {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            summary: Some("HTTP library".to_string()),
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec!["urllib3>=1.21".to_string()],
            classifiers: vec![],
        }];

        let lockfile = LockFile::from_packages(packages, "3.11".to_string());
        let converted = lockfile.to_packages();
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].name, "requests");
        assert_eq!(converted[0].version, "2.28.0");
    }

    #[test]
    fn test_lockfile_has_package() {
        let packages = vec![Package {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        }];

        let lockfile = LockFile::from_packages(packages, "3.11".to_string());
        assert!(lockfile.has_package("requests"));
        assert!(!lockfile.has_package("numpy"));
    }

    #[test]
    fn test_lockfile_validate() {
        let packages = vec![Package {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: vec![],
            classifiers: vec![],
        }];

        let lockfile = LockFile::from_packages(packages, "3.11".to_string());
        assert!(lockfile.validate().is_ok());
    }
}
