/// Default implementations of upgrade traits
use super::traits::*;
use super::detector::{self, InstalledPackage};
use super::conflict::VersionConflict;
use crate::network::get_package_metadata;
use std::cmp::Ordering;
use async_trait::async_trait;
use anyhow::Result;

/// Default package detector implementation
pub struct DefaultPackageDetector;

#[async_trait]
impl PackageDetector for DefaultPackageDetector {
    async fn get_installed(&self) -> Result<Vec<InstalledPackage>> {
        detector::get_installed_packages()
    }
    
    fn compare_versions(&self, current: &str, latest: &str) -> Ordering {
        detector::compare_versions(current, latest)
    }
}

/// Default metadata fetcher implementation
pub struct DefaultMetadataFetcher;

#[async_trait]
impl MetadataFetcher for DefaultMetadataFetcher {
    async fn fetch_latest(&self, name: &str) -> Result<String> {
        let pkg = get_package_metadata(name, "latest").await?;
        Ok(pkg.version)
    }
}

/// Default package installer implementation
pub struct DefaultPackageInstaller;

#[async_trait]
impl PackageInstaller for DefaultPackageInstaller {
    async fn upgrade(&self, name: &str, current: &str, latest: &str) -> UpgradeResult {
        crate::commands::upgrade::installer::upgrade_package(name, current, latest)
    }
    
    async fn upgrade_parallel(
        &self,
        packages: Vec<(String, String, String)>,
        concurrency: usize,
    ) -> Vec<UpgradeResult> {
        crate::commands::upgrade::installer::upgrade_packages_parallel(packages, concurrency).await
    }
}

/// Default progress reporter implementation
pub struct DefaultProgressReporter {
    #[allow(dead_code)]
    verbose: bool,
}

impl DefaultProgressReporter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl ProgressReporter for DefaultProgressReporter {
    fn report_scanning(&self, current: usize, total: usize, package: &str, is_outdated: bool) {
        let percent = (current as f64 / total as f64 * 100.0) as u32;
        let bar_width: usize = 20;
        let filled = (bar_width as f64 * percent as f64 / 100.0) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let action = if is_outdated { "Found" } else { "Scanning" };
        
        eprint!("\r[{:3}%] [{}] {}/{} | {}: {}                    ", 
            percent, bar, current, total, action, package);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }
    
    fn report_scan_complete(&self, total: usize, outdated_count: usize) {
        eprintln!("\r[100%] [{}] {}/{} | Scan complete!                                                     ", 
            "█".repeat(20), total, total);
        
        if outdated_count > 0 {
            println!("\n{:<50} {:<20} {:<20}", "Package", "Current", "Latest");
            println!("{}", "-".repeat(90));
            println!("\n✓ Found {} outdated packages to upgrade", outdated_count);
            println!("  Starting parallel upgrade (5 concurrent)...\n");
        }
    }
    
    fn report_result(&self, _result: &UpgradeResult) {
        eprint!("\r{}", " ".repeat(100));
    }
    
    fn report_conflict(&self, conflict: &VersionConflict) {
        println!("⚠️  CONFLICT: {} {} → {} conflicts with {} {}", 
            conflict.package, conflict.current_version, conflict.new_version,
            conflict.conflicting_package, conflict.conflicting_version);
        println!("   Reason: {}", conflict.reason);
    }
    
    fn report_unmet_dependencies(&self, package: &str, dependencies: &[String]) {
        if !dependencies.is_empty() {
            println!("⚠️  UNMET DEPENDENCIES for {}:", package);
            for dep in dependencies {
                println!("   - {}", dep);
            }
        }
    }
    
    fn report_summary(&self, upgraded: usize, failed: usize) {
        let separator = "=".repeat(90);
        println!("\n{}", separator);
        if failed == 0 {
            println!("✓ Upgrade complete! {} packages updated successfully", upgraded);
        } else {
            println!("⚠ Upgrade complete! {} packages updated, {} failed", upgraded, failed);
        }
        println!("{}", separator);
    }
}
