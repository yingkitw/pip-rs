/// Upgrade command handler with dependency injection
use super::traits::*;
use super::conflict::ConflictDetector;
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

        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(5));
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
                        Err(_) => {
                            // Silently skip failed requests
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

        eprintln!("\r{}", " ".repeat(100));
        eprintln!("\r[100%] [{}] {}/{} | Scan complete!", "█".repeat(20), total_packages, total_packages);

        if outdated_packages.is_empty() {
            println!("\n✓ All packages are up-to-date!\n");
            return Ok(0);
        }

        // Display outdated packages found
        println!("\n📋 Found {} outdated packages. Starting upgrade...\n", outdated_packages.len());
        println!("{:<50} {:<20} {:<20} {}", "Package", "Current", "Latest", "Status");
        println!("{}", "-".repeat(100));

        // Upgrade all packages in parallel
        let results = self.installer.upgrade_parallel(outdated_packages, self.config.concurrency).await;
        
        // Display results
        let (upgraded_count, failed_count) = results.iter().fold((0, 0), |(up, fail), result| {
            let status = if result.success { "✓ UPGRADED" } else { "✗ FAILED" };
            println!("{:<50} {:<20} {:<20} {}", 
                result.name, result.current_version, result.latest_version, status);
            
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

        fn report_result(&self, result: &UpgradeResult) {
            self.results
                .lock()
                .unwrap()
                .push(format!("{}: {}", result.name, result.success));
        }

        fn report_conflict(&self, _conflict: &super::super::conflict::VersionConflict) {}

        fn report_unmet_dependencies(&self, _package: &str, _dependencies: &[String]) {}

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

        assert_eq!(handler.config.concurrency, 5);
    }
}
