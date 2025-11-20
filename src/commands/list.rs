/// List command implementation
use anyhow::Result;
use std::path::Path;
use std::fs;
use std::cmp::Ordering;

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
                                packages.push(Package { name: pkg_name, version });
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
        // ... (rest of the function)
    }

    Ok(0)
}
