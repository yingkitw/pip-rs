/// Traits for dependency injection and testability
use async_trait::async_trait;
use anyhow::Result;
use std::cmp::Ordering;

use super::detector::InstalledPackage;
use super::conflict::{VersionConflict, ConflictDetector};

/// Trait for package detection
#[async_trait]
pub trait PackageDetector: Send + Sync {
    /// Get installed packages
    async fn get_installed(&self) -> Result<Vec<InstalledPackage>>;
    
    /// Compare two versions
    fn compare_versions(&self, current: &str, latest: &str) -> Ordering;
}

/// Trait for package metadata fetching
#[async_trait]
pub trait MetadataFetcher: Send + Sync {
    /// Fetch latest version for a package
    async fn fetch_latest(&self, name: &str) -> Result<String>;
}

/// Trait for package installation
#[async_trait]
pub trait PackageInstaller: Send + Sync {
    /// Upgrade a single package
    #[allow(dead_code)]
    async fn upgrade(&self, name: &str, current: &str, latest: &str) -> UpgradeResult;
    
    /// Upgrade multiple packages in parallel
    async fn upgrade_parallel(
        &self,
        packages: Vec<(String, String, String)>,
        concurrency: usize,
    ) -> Vec<UpgradeResult>;
}

/// Trait for progress reporting
pub trait ProgressReporter: Send + Sync {
    /// Report scanning progress
    fn report_scanning(&self, current: usize, total: usize, package: &str, is_outdated: bool);
    
    /// Report scanning complete
    fn report_scan_complete(&self, total: usize, outdated_count: usize);
    
    /// Report upgrade result
    fn report_result(&self, result: &UpgradeResult);
    
    /// Report version conflict
    fn report_conflict(&self, conflict: &VersionConflict);
    
    /// Report unmet dependencies
    fn report_unmet_dependencies(&self, package: &str, dependencies: &[String]);
    
    /// Report final summary
    fn report_summary(&self, upgraded: usize, failed: usize);
}

/// Upgrade result
#[derive(Clone, Debug)]
pub struct UpgradeResult {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub success: bool,
    #[allow(dead_code)]
    pub error_msg: Option<String>,
}

/// Upgrade command configuration
#[derive(Clone)]
pub struct UpgradeConfig {
    pub concurrency: usize,
    #[allow(dead_code)]
    pub verbose: bool,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            concurrency: 5,
            verbose: false,
        }
    }
}
