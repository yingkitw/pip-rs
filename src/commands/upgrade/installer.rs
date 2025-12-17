/// Package installation/upgrade functionality
use std::process::Command;
use super::traits::UpgradeResult;

/// Fast upgrade using pip-rs native installation (no subprocess overhead)
pub async fn upgrade_package_fast(name: &str, _current: &str, latest: &str) -> UpgradeResult {
    // Use pip-rs native installation for maximum speed
    use crate::commands::install::handle_install;
    
    let package_spec = format!("{}=={}", name, latest);
    match handle_install(vec![package_spec], None, None, Vec::new(), None).await {
        Ok(_) => UpgradeResult {
            name: name.to_string(),
            current_version: _current.to_string(),
            latest_version: latest.to_string(),
            success: true,
            error_msg: None,
        },
        Err(e) => UpgradeResult {
            name: name.to_string(),
            current_version: _current.to_string(),
            latest_version: latest.to_string(),
            success: false,
            error_msg: Some(format!("{}", e)),
        },
    }
}

/// Legacy upgrade using pip subprocess (kept for compatibility)
#[allow(dead_code)]
pub fn upgrade_package(name: &str, current: &str, latest: &str) -> UpgradeResult {
    let package_spec = format!("{}=={}", name, latest);
    
    let output = Command::new("pip")
        .args(&["install", "--upgrade", &package_spec, "-q"])
        .output();
    
    match output {
        Ok(result) => {
            if !result.status.success() {
                return UpgradeResult {
                    name: name.to_string(),
                    current_version: current.to_string(),
                    latest_version: latest.to_string(),
                    success: false,
                    error_msg: Some("Installation failed".to_string()),
                };
            }
            
            // Skip slow version verification for speed
            UpgradeResult {
                name: name.to_string(),
                current_version: current.to_string(),
                latest_version: latest.to_string(),
                success: true,
                error_msg: None,
            }
        }
        Err(e) => {
            UpgradeResult {
                name: name.to_string(),
                current_version: current.to_string(),
                latest_version: latest.to_string(),
                success: false,
                error_msg: Some(e.to_string()),
            }
        }
    }
}

/// Fast batch upgrade - installs multiple packages in one pip command
/// This is the fastest method: single pip install --upgrade command for all packages
pub async fn upgrade_packages_batch(
    packages: Vec<(String, String, String)>,
) -> Vec<UpgradeResult> {
    if packages.is_empty() {
        return Vec::new();
    }
    
    // Build package specs for batch installation
    let package_specs: Vec<String> = packages
        .iter()
        .map(|(name, _, latest)| format!("{}=={}", name, latest))
        .collect();
    
    // Use pip subprocess for reliable batch upgrade - much faster than individual installs
    let mut args = vec!["install", "--upgrade", "-q"];
    let spec_refs: Vec<&str> = package_specs.iter().map(|s| s.as_str()).collect();
    args.extend(spec_refs);
    
    let output = Command::new("pip")
        .args(&args)
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            // All packages succeeded in batch - fastest path
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
        _ => {
            // Batch failed - fall back to parallel individual upgrades
            upgrade_packages_parallel_pip(packages, 10).await
        }
    }
}

/// Parallel upgrade using pip subprocess - reliable fallback
pub async fn upgrade_packages_parallel_pip(
    packages: Vec<(String, String, String)>,
    concurrency: usize,
) -> Vec<UpgradeResult> {
    use std::sync::Arc;
    use tokio::sync::{Semaphore, Mutex};
    use futures::future::join_all;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let total = packages.len();
    let completed = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    for (name, current, latest) in packages {
        let sem = semaphore.clone();
        let completed_clone = completed.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.ok();
            
            // Use pip subprocess for each package
            let package_spec = format!("{}=={}", name, latest);
            let output = Command::new("pip")
                .args(&["install", "--upgrade", "-q", &package_spec])
                .output();
            
            let result = match output {
                Ok(result) if result.status.success() => UpgradeResult {
                    name: name.clone(),
                    current_version: current,
                    latest_version: latest,
                    success: true,
                    error_msg: None,
                },
                Ok(result) => UpgradeResult {
                    name: name.clone(),
                    current_version: current,
                    latest_version: latest,
                    success: false,
                    error_msg: Some(String::from_utf8_lossy(&result.stderr).to_string()),
                },
                Err(e) => UpgradeResult {
                    name: name.clone(),
                    current_version: current,
                    latest_version: latest,
                    success: false,
                    error_msg: Some(e.to_string()),
                },
            };
            
            // Update progress
            let mut count = completed_clone.lock().await;
            *count += 1;
            let current_count = *count;
            drop(count);
            
            if current_count % 10 == 0 || current_count == total {
                eprint!("\r  Upgraded {}/{} packages...", current_count, total);
                let _ = std::io::Write::flush(&mut std::io::stderr());
            }
            
            result
        });
        handles.push(handle);
    }

    let results = join_all(handles).await;
    eprintln!("\r  Upgraded {}/{} packages...✓", total, total);
    results.into_iter().filter_map(|r| r.ok()).collect()
}

/// Legacy parallel upgrade using pip subprocess (kept for compatibility)
#[allow(dead_code)]
pub async fn upgrade_packages_parallel(
    packages: Vec<(String, String, String)>,
    concurrency: usize,
) -> Vec<UpgradeResult> {
    upgrade_packages_parallel_pip(packages, concurrency).await
}
