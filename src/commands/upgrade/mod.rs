/// Package upgrade command with modular components
pub mod progress;
pub mod detector;
pub mod installer;

use anyhow::Result;
use crate::network::get_package_metadata;
use std::cmp::Ordering;
use std::sync::Arc;
use std::io::Write;
use tokio::sync::{Semaphore, mpsc};
use futures::future::join_all;

use detector::{get_installed_packages, compare_versions};
use progress::ProgressIndicator;
use installer::upgrade_package;

#[allow(dead_code)]
pub async fn handle_upgrade(package_name: &str, _target: Option<&str>) -> Result<()> {
    println!("Checking for updates for package: {}", package_name);

    // Get current version (would come from installed packages)
    let current_version = "1.0.0"; // Placeholder

    // Fetch latest version from PyPI
    let latest_package = get_package_metadata(package_name, "latest").await?;

    // Compare versions
    if compare_versions(current_version, &latest_package.version) == Ordering::Less {
        println!(
            "Upgrade available: {} -> {}",
            current_version, latest_package.version
        );
        println!("Run 'pip install --upgrade {}' to upgrade", package_name);
    } else {
        println!("Package {} is already at the latest version", package_name);
    }

    Ok(())
}

pub async fn handle_upgrade_all() -> Result<i32> {
    println!("Checking for updates and upgrading packages...");
    
    // Get installed packages
    let packages = get_installed_packages()?;
    
    if packages.is_empty() {
        println!("No packages found in site-packages");
        return Ok(0);
    }

    // Sort packages by name
    let mut packages = packages;
    packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    println!("\n{:<50} {:<20} {:<20} {}", "Package", "Current", "Latest", "Status");
    println!("{}", "-".repeat(100));
    
    // Create channel for real-time result streaming
    let (tx, mut rx) = mpsc::channel(100);
    let total_packages = packages.len();
    
    // Spawn task to fetch packages
    let packages_clone = packages.clone();
    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(5));
        let mut handles = vec![];
        
        // Spawn all tasks at once for real-time streaming
        for pkg in packages_clone.iter() {
            let semaphore_clone = semaphore.clone();
            let tx_clone = tx.clone();
            let name = pkg.name.clone();
            let version = pkg.version.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.ok();
                match get_package_metadata(&name, "latest").await {
                    Ok(pkg_info) => {
                        let is_outdated = compare_versions(&version, &pkg_info.version) == Ordering::Less;
                        let _ = tx_clone.send((name, version, pkg_info.version, is_outdated)).await;
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
    
    // Collect outdated packages and show checking progress with real-time upgrades
    let mut outdated_packages = Vec::new();
    let mut checked_count = 0;
    let mut upgraded_count = 0;
    let mut failed_count = 0;
    
    println!("\nScanning and upgrading packages in real-time:\n");
    
    while let Some((name, version, latest, is_outdated)) = rx.recv().await {
        checked_count += 1;
        
        if is_outdated {
            // Show and upgrade immediately
            eprint!("\r[{}/{}] Upgrading: {} ({} → {})...", 
                checked_count, total_packages, name, version, latest);
            let _ = std::io::stderr().flush();
            
            // Upgrade immediately
            let result = upgrade_package(&name, &version, &latest);
            
            if result.success {
                upgraded_count += 1;
                println!("\r{:<50} {:<20} {:<20} ✓ UPGRADED", name, version, latest);
            } else {
                failed_count += 1;
                println!("\r{:<50} {:<20} {:<20} ✗ FAILED", name, version, latest);
            }
            
            outdated_packages.push((name, version, latest));
        } else {
            // Just show scanning progress for up-to-date
            eprint!("\r[{}/{}] Scanning: {} (up-to-date)...", checked_count, total_packages, name);
            let _ = std::io::stderr().flush();
        }
        
        // Break if all packages checked
        if checked_count >= total_packages {
            break;
        }
    }
    
    eprintln!("\r[{}/{}] Scanning complete!", total_packages, total_packages);
    
    if outdated_packages.is_empty() {
        println!("\nAll packages are up-to-date!");
        return Ok(0);
    }
    
    
    // Summary
    println!("\n{}", "=".repeat(100));
    println!("Upgrade complete! {} packages updated, {} failed", upgraded_count, failed_count);
    
    Ok(upgraded_count as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upgrade_command() -> Result<()> {
        // This would require mocking the network calls
        Ok(())
    }
}
