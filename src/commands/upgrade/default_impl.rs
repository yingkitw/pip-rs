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
        // For version checking, we need fresh data from PyPI, not cached
        // Use get_latest_version which bypasses cache for accurate outdated detection
        use crate::network::pypi::get_latest_version;
        get_latest_version(name).await
    }
}

/// Default package installer implementation
pub struct DefaultPackageInstaller;

#[async_trait]
impl PackageInstaller for DefaultPackageInstaller {
    async fn upgrade(&self, name: &str, current: &str, latest: &str) -> UpgradeResult {
        // Use fast native upgrade (async)
        crate::commands::upgrade::installer::upgrade_package_fast(name, current, latest).await
    }
    
    async fn upgrade_parallel(
        &self,
        packages: Vec<(String, String, String)>,
        _concurrency: usize,
    ) -> Vec<UpgradeResult> {
        // Try batch upgrade first (fastest), fall back to parallel if needed
        crate::commands::upgrade::installer::upgrade_packages_batch(packages).await
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
        let bar_width: usize = 25;
        let filled = (bar_width as f64 * percent as f64 / 100.0) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        
        // Truncate package name to fit terminal width
        let pkg_display = if package.len() > 30 {
            format!("{}...", &package[..27])
        } else {
            package.to_string()
        };
        
        let status_icon = if is_outdated { "🔄" } else { "✓" };
        
        eprint!("\r  {status_icon} [{:3}%] [{bar}] {current:>4}/{total:<4} | {pkg_display:<33}", 
            percent, status_icon = status_icon, bar = bar, current = current, total = total, pkg_display = pkg_display);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }
    
    fn report_scan_complete(&self, total: usize, outdated_count: usize) {
        eprintln!("\r  ✓ [100%] [{}] {}/{} | Scan complete!                                    ", 
            "█".repeat(25), total, total);
        
        if outdated_count > 0 {
            println!("\n  📦 Found {outdated_count} outdated package{} to upgrade", 
                if outdated_count == 1 { "" } else { "s" });
            println!("  ⚡ Starting fast batch upgrade...\n");
            println!("  {:<45} {:<15} {:<15} {:<12}", "Package", "Current", "Latest", "Status");
            println!("  {}", "-".repeat(90));
        }
    }
    
    fn report_result(&self, _result: &UpgradeResult) {
        // Results are printed directly in handler, no need to print here
    }
    
    fn report_conflict(&self, conflict: &VersionConflict) {
        println!("  ⚠️  CONFLICT: {} {} → {} conflicts with {} {}", 
            conflict.package, conflict.current_version, conflict.new_version,
            conflict.conflicting_package, conflict.conflicting_version);
        println!("      Reason: {}", conflict.reason);
    }
    
    fn report_unmet_dependencies(&self, package: &str, dependencies: &[String]) {
        if !dependencies.is_empty() {
            println!("  ⚠️  UNMET DEPENDENCIES for {}:", package);
            for dep in dependencies {
                println!("      - {}", dep);
            }
        }
    }
    
    fn report_summary(&self, upgraded: usize, failed: usize) {
        let separator = "  ".to_string() + &"─".repeat(88);
        println!("\n{}", separator);
        if failed == 0 {
            println!("  ✅ Success! {} package{} updated", upgraded, if upgraded == 1 { "" } else { "s" });
        } else {
            println!("  ⚠️  Completed with issues: {} updated, {} failed", upgraded, failed);
        }
        println!("{}\n", separator);
    }
}
