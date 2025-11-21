/// List command implementation
use anyhow::Result;
use std::path::Path;
use std::fs;
use std::cmp::Ordering;
use std::io::Write;

#[derive(Debug, Clone)]
struct Package {
    name: String,
    version: String,
    latest_version: Option<String>,
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

use crate::errors::PipError;

// ... (rest of the file)

pub async fn handle_list(outdated: bool) -> Result<i32, PipError> {
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
            let entries = fs::read_dir(path).map_err(|e| PipError::FileSystemError {
                path: path.to_string_lossy().to_string(),
                operation: "read".to_string(),
                reason: e.to_string(),
            })?;
            for entry in entries {
                let entry = entry.map_err(|e| PipError::FileSystemError {
                    path: "site-packages".to_string(),
                    operation: "read entry".to_string(),
                    reason: e.to_string(),
                })?;
                let entry_path = entry.path();
                if let Some(name) = entry_path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.ends_with(".dist-info") {
                            // Parse package name and version
                            let pkg_info = name_str.trim_end_matches(".dist-info");
                            if let Some(last_dash) = pkg_info.rfind('-') {
                                let pkg_name = pkg_info[..last_dash].to_string();
                                let version = pkg_info[last_dash + 1..].to_string();
                                packages.push(Package { 
                                    name: pkg_name, 
                                    version,
                                    latest_version: None,
                                });
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
        return Ok(0);
    }

    // Sort packages by name
    packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // If outdated flag is set, fetch latest versions
    if outdated {
        use crate::network::get_package_metadata;
        use std::sync::Arc;
        use tokio::sync::Semaphore;
        use futures::future::join_all;

        println!("\n📦 Checking for updates ({} packages)...\n", packages.len());

        // Fetch latest versions in parallel (10 concurrent)
        let semaphore = Arc::new(Semaphore::new(10));
        let mut handles = vec![];
        let total = packages.len();

        for (idx, pkg) in packages.iter_mut().enumerate() {
            let semaphore_clone = semaphore.clone();
            let pkg_name = pkg.name.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.ok();
                match get_package_metadata(&pkg_name, "latest").await {
                    Ok(metadata) => Some((pkg_name, metadata.version, idx)),
                    Err(_) => None,
                }
            });
            handles.push(handle);
        }

        // Collect results with progress feedback
        let results = join_all(handles).await;
        let mut checked = 0;
        for result in results.into_iter() {
            checked += 1;
            if checked % 50 == 0 || checked == total {
                eprint!("\r  Checked {}/{} packages...", checked, total);
                let _ = std::io::Write::flush(&mut std::io::stderr());
            }
            
            if let Ok(Some((_, latest, idx))) = result {
                packages[idx].latest_version = Some(latest);
            }
        }
        eprintln!("\r  Checked {}/{} packages...✓", total, total);

        // Filter to only outdated packages
        packages.retain(|pkg| {
            if let Some(latest) = &pkg.latest_version {
                compare_versions(&pkg.version, latest) == Ordering::Less
            } else {
                false
            }
        });

        if packages.is_empty() {
            println!("✓ All packages are up-to-date!\n");
            return Ok(0);
        }

        // Display outdated packages
        println!("{:<45} {:<15} {:<15}", "Package", "Current", "Latest");
        println!("{}", "-".repeat(75));
        
        for pkg in packages {
            if let Some(latest) = pkg.latest_version {
                println!("{:<45} {:<15} {:<15}", pkg.name, pkg.version, latest);
            }
        }
    } else {
        // Display all packages
        println!("\n{:<50} {:<20}", "Package", "Version");
        println!("{}", "-".repeat(70));
        
        for pkg in packages {
            println!("{:<50} {:<20}", pkg.name, pkg.version);
        }
    }
    
    println!();
    Ok(0)
}
