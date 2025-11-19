/// List command implementation
use anyhow::Result;
use std::path::Path;
use std::fs;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use futures::future::join_all;

#[derive(Debug, Clone)]
struct Package {
    name: String,
    version: String,
}

fn compare_versions(current: &str, latest: &str) -> Ordering {
    let current_parts: Vec<&str> = current.split('.').collect();
    let latest_parts: Vec<&str> = latest.split('.').collect();

    for i in 0..current_parts.len().max(latest_parts.len()) {
        let curr = current_parts.get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let lat = latest_parts.get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        match curr.cmp(&lat) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

pub async fn handle_list(outdated: bool) -> Result<i32> {
    // Check common site-packages locations
    let site_packages_paths = vec![
        // macOS with Python.org installer
        "/Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages",
        "/Library/Frameworks/Python.framework/Versions/3.11/lib/python3.11/site-packages",
        "/Library/Frameworks/Python.framework/Versions/3.10/lib/python3.10/site-packages",
        // Linux
        "/usr/local/lib/python3.12/site-packages",
        "/usr/local/lib/python3.11/site-packages",
        "/usr/lib/python3/dist-packages",
        // User site-packages
        "~/.local/lib/python3.12/site-packages",
        "~/.local/lib/python3.11/site-packages",
    ];

    let mut packages = Vec::new();

    for path_str in site_packages_paths {
        let expanded_path = if path_str.starts_with('~') {
            shellexpand::tilde(path_str).to_string()
        } else {
            path_str.to_string()
        };

        let path = Path::new(&expanded_path);
        if path.exists() {
            // List .dist-info directories
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if let Some(name) = entry_path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if name_str.ends_with(".dist-info") {
                                // Parse package name and version
                                let pkg_info = name_str.trim_end_matches(".dist-info");
                                if let Some(last_dash) = pkg_info.rfind('-') {
                                    let pkg_name = pkg_info[..last_dash].to_string();
                                    let version = pkg_info[last_dash + 1..].to_string();
                                    packages.push(Package { name: pkg_name, version });
                                }
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    if packages.is_empty() {
        println!("No packages found in site-packages");
    } else {
        // Sort packages by name
        packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        if outdated {
            // Format: Package Version Latest Type
            println!("{:<50} {:<20} {:<20} {}", "Package", "Version", "Latest", "Type");
            println!("{}", "-".repeat(100));
            
            eprintln!("Checking {} packages for updates (real-time streaming)...", packages.len());
            
            // Create channel for real-time result streaming
            let (tx, mut rx) = mpsc::channel(100);
            let total_packages = packages.len();
            
            // Spawn task to fetch packages
            let packages_clone = packages.clone();
            tokio::spawn(async move {
                let semaphore = Arc::new(Semaphore::new(5));
                let mut handles = vec![];
                
                // Spawn all tasks at once for real-time streaming
                for (_idx, pkg) in packages_clone.iter().enumerate() {
                    let semaphore_clone = semaphore.clone();
                    let tx_clone = tx.clone();
                    let name = pkg.name.clone();
                    let version = pkg.version.clone();
                    
                    let handle = tokio::spawn(async move {
                        let _permit = semaphore_clone.acquire().await.ok();
                        match crate::network::get_package_metadata(&name, "latest").await {
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
            
            // Receive and display results in real-time
            let mut outdated_count = 0;
            let mut checked_count = 0;
            
            while let Some((name, version, latest, is_outdated)) = rx.recv().await {
                checked_count += 1;
                
                if is_outdated {
                    outdated_count += 1;
                    println!("{:<50} {:<20} {:<20} {}", name, version, latest, "wheel");
                }
                
                // Show progress every 100 packages
                if checked_count % 100 == 0 {
                    eprintln!("Progress: {}/{} packages checked", checked_count, total_packages);
                }
                
                // Break if all packages checked
                if checked_count >= total_packages {
                    break;
                }
            }
            
            eprintln!("Progress: {}/{} packages checked", total_packages, total_packages);
            println!("\nTotal: {} outdated packages", outdated_count);
        } else {
            // Format: Package Version
            println!("{:<50} {}", "Package", "Version");
            println!("{}", "-".repeat(70));
            
            for pkg in &packages {
                println!("{:<50} {}", pkg.name, pkg.version);
            }
            
            println!("\nTotal: {} packages", packages.len());
        }
    }

    Ok(0)
}
