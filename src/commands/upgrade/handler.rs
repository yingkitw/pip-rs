/// Upgrade command handler with dependency injection
use super::traits::*;
use anyhow::Result;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use futures::future::join_all;

/// Upgrade handler with injectable dependencies
pub struct UpgradeHandler<D, M, I, P>
where
    D: PackageDetector + 'static,
    M: MetadataFetcher + 'static,
    I: PackageInstaller + 'static,
    P: ProgressReporter + 'static,
{
    detector: Arc<D>,
    fetcher: Arc<M>,
    installer: Arc<I>,
    reporter: Arc<P>,
    config: UpgradeConfig,
}

impl<D, M, I, P> UpgradeHandler<D, M, I, P>
where
    D: PackageDetector + 'static,
    M: MetadataFetcher + 'static,
    I: PackageInstaller + 'static,
    P: ProgressReporter + 'static,
{
    /// Create a new upgrade handler
    pub fn new(
        detector: D,
        fetcher: M,
        installer: I,
        reporter: P,
        config: UpgradeConfig,
    ) -> Self {
        Self {
            detector: Arc::new(detector),
            fetcher: Arc::new(fetcher),
            installer: Arc::new(installer),
            reporter: Arc::new(reporter),
            config,
        }
    }

    /// Execute upgrade for all outdated packages
    pub async fn upgrade_all(&self) -> Result<i32> {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║           pip-rs Package Update Tool                           ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        // Get installed packages
        let packages = self.detector.get_installed().await?;

        if packages.is_empty() {
            println!("✗ No packages found in site-packages");
            return Ok(0);
        }

        // Sort packages by name
        let mut packages = packages;
        packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        println!("📦 Scanning {} installed packages for updates...\n", packages.len());

        // Create channel for real-time result streaming
        let (tx, mut rx) = mpsc::channel(100);
        let total_packages = packages.len();

        // Spawn task to fetch packages
        let packages_clone = packages.clone();
        let fetcher = self.fetcher.clone();
        let detector = self.detector.clone();

        let scan_task = tokio::spawn(async move {
            // Reduced to 15 to avoid PyPI rate limiting while still being fast
            let semaphore = Arc::new(Semaphore::new(15));
            let mut handles = vec![];

            // Spawn all tasks at once for real-time streaming
            for pkg in packages_clone.iter() {
                let semaphore_clone = semaphore.clone();
                let tx_clone = tx.clone();
                let name = pkg.name.clone();
                let version = pkg.version.clone();
                let fetcher_clone = fetcher.clone();
                let detector_clone = detector.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.ok();
                    match fetcher_clone.fetch_latest(&name).await {
                        Ok(latest) => {
                            let is_outdated =
                                detector_clone.compare_versions(&version, &latest) == Ordering::Less;
                            let _ = tx_clone.send((name, version, latest, is_outdated)).await;
                        }
                        Err(e) => {
                            // Log error in debug mode only, don't spam stderr
                            tracing::debug!("Failed to fetch latest version for {}: {}", name, e);
                            // Send a dummy message to indicate task completed
                            let _ = tx_clone.send((String::new(), String::new(), String::new(), false)).await;
                        }
                    }
                });
                handles.push(handle);
            }

            // Wait for all tasks to complete
            let _ = join_all(handles).await;
        });

        // Collect all outdated packages first
        let mut checked_count = 0;
        let mut outdated_packages = Vec::new();
        
        while let Some((name, version, latest, is_outdated)) = rx.recv().await {
            // Skip empty messages from failed requests
            if name.is_empty() {
                checked_count += 1;
                continue;
            }
            
            checked_count += 1;
            
            if is_outdated {
                outdated_packages.push((name, version, latest));
            } else {
                self.reporter
                    .report_scanning(checked_count, total_packages, &name, false);
            }

            // Break if all packages checked
            if checked_count >= total_packages {
                break;
            }
        }

        // Ensure the scan task completes
        let _ = scan_task.await;

        eprintln!("\r{}", " ".repeat(100));

        if outdated_packages.is_empty() {
            println!("\n  ✓ All packages are up-to-date!\n");
            return Ok(0);
        }

        // Display outdated packages found
        self.reporter.report_scan_complete(packages.len(), outdated_packages.len());

        // Fast batch upgrade - installs all packages in one command for maximum speed
        println!("  ⚡ Upgrading {} packages using fast batch installation...\n", outdated_packages.len());
        let results = self.installer.upgrade_parallel(outdated_packages, self.config.concurrency).await;
        
        // Display results with better formatting
        let (upgraded_count, failed_count) = results.iter().fold((0, 0), |(up, fail), result| {
            let status_icon = if result.success { "✅" } else { "❌" };
            let status_text = if result.success { "UPGRADED" } else { "FAILED" };
            
            // Truncate package name if too long
            let pkg_name = if result.name.len() > 45 {
                format!("{}...", &result.name[..42])
            } else {
                result.name.clone()
            };
            
            println!("  {status_icon} {:<45} {:<15} {:<15} {}", 
                pkg_name, result.current_version, result.latest_version, status_text);
            
            if result.success {
                (up + 1, fail)
            } else {
                (up, fail + 1)
            }
        });

        self.reporter.report_summary(upgraded_count, failed_count);
        Ok(upgraded_count as i32)
    }

    /// Execute upgrade for specific packages
    pub async fn upgrade_packages(&self, packages_to_upgrade: Vec<String>) -> Result<i32> {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║           pip-rs Package Update Tool                           ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        // Get installed packages
        let installed_packages = self.detector.get_installed().await?;

        if installed_packages.is_empty() {
            println!("✗ No packages found in site-packages");
            return Ok(0);
        }

        // Filter to only packages requested (normalize names for comparison)
        let packages: Vec<_> = installed_packages
            .into_iter()
            .filter(|p| {
                let normalized_name = p.name.to_lowercase().replace('_', "-");
                packages_to_upgrade.iter().any(|req| {
                    let normalized_req = req.to_lowercase().replace('_', "-");
                    normalized_name == normalized_req
                })
            })
            .collect();

        if packages.is_empty() {
            println!("✗ None of the requested packages are installed. Nothing to do.");
            return Ok(0);
        }
        
        println!("📦 Scanning {} requested packages for updates...\n", packages.len());

        // Create channel for real-time result streaming
        let (tx, mut rx) = mpsc::channel(100);
        let total_packages = packages.len();

        // Spawn task to fetch packages
        let packages_clone = packages.clone();
        let fetcher = self.fetcher.clone();
        let detector = self.detector.clone();

        let scan_task = tokio::spawn(async move {
            // Reduced to 15 to avoid PyPI rate limiting while still being fast
            let semaphore = Arc::new(Semaphore::new(15));
            let mut handles = vec![];

            // Spawn all tasks at once for real-time streaming
            for pkg in packages_clone.iter() {
                let semaphore_clone = semaphore.clone();
                let tx_clone = tx.clone();
                let name = pkg.name.clone();
                let version = pkg.version.clone();
                let fetcher_clone = fetcher.clone();
                let detector_clone = detector.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.ok();
                    match fetcher_clone.fetch_latest(&name).await {
                        Ok(latest) => {
                            let is_outdated =
                                detector_clone.compare_versions(&version, &latest) == Ordering::Less;
                            let _ = tx_clone.send((name, version, latest, is_outdated)).await;
                        }
                        Err(e) => {
                            // Log error in debug mode only, don't spam stderr
                            tracing::debug!("Failed to fetch latest version for {}: {}", name, e);
                            // Send a dummy message to indicate task completed
                            let _ = tx_clone.send((String::new(), String::new(), String::new(), false)).await;
                        }
                    }
                });
                handles.push(handle);
            }

            // Wait for all tasks to complete
            let _ = join_all(handles).await;
        });

        // Collect all outdated packages first
        let mut checked_count = 0;
        let mut outdated_packages = Vec::new();
        
        while let Some((name, version, latest, is_outdated)) = rx.recv().await {
            // Skip empty messages from failed requests
            if name.is_empty() {
                checked_count += 1;
                continue;
            }
            
            checked_count += 1;
            
            if is_outdated {
                outdated_packages.push((name, version, latest));
            } else {
                self.reporter
                    .report_scanning(checked_count, total_packages, &name, false);
            }

            // Break if all packages checked
            if checked_count >= total_packages {
                break;
            }
        }

        // Ensure the scan task completes
        let _ = scan_task.await;

        eprintln!("\r{}", " ".repeat(100));

        if outdated_packages.is_empty() {
            println!("\n  ✓ All requested packages are up-to-date!\n");
            return Ok(0);
        }

        // Display outdated packages found
        self.reporter.report_scan_complete(packages.len(), outdated_packages.len());

        // Fast batch upgrade - installs all packages in one command for maximum speed
        println!("  ⚡ Upgrading {} packages using fast batch installation...\n", outdated_packages.len());
        let results = self.installer.upgrade_parallel(outdated_packages, self.config.concurrency).await;
        
        // Display results with better formatting
        let (upgraded_count, failed_count) = results.iter().fold((0, 0), |(up, fail), result| {
            let status_icon = if result.success { "✅" } else { "❌" };
            let status_text = if result.success { "UPGRADED" } else { "FAILED" };
            
            // Truncate package name if too long
            let pkg_name = if result.name.len() > 45 {
                format!("{}...", &result.name[..42])
            } else {
                result.name.clone()
            };
            
            println!("  {status_icon} {:<45} {:<15} {:<15} {}", 
                pkg_name, result.current_version, result.latest_version, status_text);
            
            if result.success {
                (up + 1, fail)
            } else {
                (up, fail + 1)
            }
        });

        self.reporter.report_summary(upgraded_count, failed_count);
        Ok(upgraded_count as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::upgrade::detector::InstalledPackage;
    use std::sync::Mutex;

    /// Mock detector for testing
    struct MockDetector {
        packages: Vec<InstalledPackage>,
    }

    #[async_trait::async_trait]
    impl PackageDetector for MockDetector {
        async fn get_installed(&self) -> Result<Vec<InstalledPackage>> {
            Ok(self.packages.clone())
        }

        fn compare_versions(&self, current: &str, latest: &str) -> Ordering {
            current.cmp(latest)
        }
    }

    /// Mock fetcher for testing
    struct MockFetcher {
        versions: std::collections::HashMap<String, String>,
    }

    #[async_trait::async_trait]
    impl MetadataFetcher for MockFetcher {
        async fn fetch_latest(&self, name: &str) -> Result<String> {
            Ok(self
                .versions
                .get(name)
                .cloned()
                .unwrap_or_else(|| "1.0.0".to_string()))
        }
    }

    /// Mock installer for testing
    struct MockInstaller;

    #[async_trait::async_trait]
    impl PackageInstaller for MockInstaller {
        async fn upgrade(&self, name: &str, current: &str, latest: &str) -> UpgradeResult {
            UpgradeResult {
                name: name.to_string(),
                current_version: current.to_string(),
                latest_version: latest.to_string(),
                success: true,
                error_msg: None,
            }
        }

        async fn upgrade_parallel(
            &self,
            packages: Vec<(String, String, String)>,
            _concurrency: usize,
        ) -> Vec<UpgradeResult> {
            packages
                .into_iter()
                .map(|(name, current, latest)| UpgradeResult {
                    name,
                    current_version: current,
                    latest_version: latest,
                    success: true,
                    error_msg: None,
                })
                .collect()
        }
    }

    /// Mock reporter for testing
    struct MockReporter {
        results: Mutex<Vec<String>>,
    }

    impl ProgressReporter for MockReporter {
        fn report_scanning(&self, _current: usize, _total: usize, _package: &str, _is_outdated: bool) {
        }

        fn report_scan_complete(&self, _total: usize, _outdated_count: usize) {}

        fn report_summary(&self, _upgraded: usize, _failed: usize) {}
    }

    #[tokio::test]
    async fn test_upgrade_handler_creation() {
        let detector = MockDetector {
            packages: vec![],
        };
        let fetcher = MockFetcher {
            versions: std::collections::HashMap::new(),
        };
        let installer = MockInstaller;
        let reporter = MockReporter {
            results: Mutex::new(Vec::new()),
        };

        let handler = UpgradeHandler::new(
            detector,
            fetcher,
            installer,
            reporter,
            UpgradeConfig::default(),
        );

        assert_eq!(handler.config.concurrency, 15);
    }
}
