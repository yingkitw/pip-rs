/// Package detection and version comparison
use anyhow::Result;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

pub fn compare_versions(current: &str, latest: &str) -> Ordering {
    // Use PEP 440 version parsing for proper comparison
    match (pep440::Version::parse(current), pep440::Version::parse(latest)) {
        (Some(v1), Some(v2)) => v1.cmp(&v2),
        // Fallback to string comparison if parsing fails
        _ => {
            // Simple fallback: try numeric comparison on first parts
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
    }
}

/// Fast package detection using pip-rs site-packages handler
pub fn get_installed_packages() -> Result<Vec<InstalledPackage>> {
    use crate::installer::site_packages::SitePackages;
    
    // Use pip-rs native site-packages detection (faster, auto-detects venv)
    match SitePackages::default() {
        Ok(site_packages) => {
            match site_packages.get_installed_packages() {
                Ok(_pkg_names) => {
                    // Fast path: parse versions directly from .dist-info directories
                    // This is faster than calling get_installed_packages() which does extra work
                    let mut packages = Vec::new();
                    let site_path = site_packages.path();
                    
                    if let Ok(entries) = fs::read_dir(site_path) {
                        for entry in entries.flatten() {
                            let entry_path = entry.path();
                            if let Some(name) = entry_path.file_name() {
                                if let Some(name_str) = name.to_str() {
                                    if name_str.ends_with(".dist-info") {
                                        let pkg_info = name_str.trim_end_matches(".dist-info");
                                        
                                        // Find the version part (starts with a digit)
                                        let mut version_start = pkg_info.len();
                                        for (i, ch) in pkg_info.char_indices().rev() {
                                            if ch == '-' && i + 1 < pkg_info.len() {
                                                if let Some(next_ch) = pkg_info[i + 1..].chars().next() {
                                                    if next_ch.is_ascii_digit() {
                                                        version_start = i;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        
                                        if version_start < pkg_info.len() {
                                            let pkg_name = pkg_info[..version_start].to_string();
                                            let version = pkg_info[version_start + 1..].to_string();
                                            packages.push(InstalledPackage { name: pkg_name, version });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(packages)
                }
                Err(_) => get_installed_packages_fallback(),
            }
        }
        Err(_) => get_installed_packages_fallback(),
    }
}

/// Fallback package detection for compatibility
fn get_installed_packages_fallback() -> Result<Vec<InstalledPackage>> {
    let mut packages = Vec::new();
    
    // Common site-packages locations (macOS, Linux, Windows)
    let site_packages_paths = vec![
        // macOS
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
                                let pkg_info = name_str.trim_end_matches(".dist-info");
                                
                                // Find the version part (starts with a digit)
                                let mut version_start = pkg_info.len();
                                for (i, ch) in pkg_info.char_indices().rev() {
                                    if ch == '-' && i + 1 < pkg_info.len() {
                                        if let Some(next_ch) = pkg_info[i + 1..].chars().next() {
                                            if next_ch.is_ascii_digit() {
                                                version_start = i;
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                if version_start < pkg_info.len() {
                                    let pkg_name = pkg_info[..version_start].to_string();
                                    let version = pkg_info[version_start + 1..].to_string();
                                    packages.push(InstalledPackage { name: pkg_name, version });
                                }
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    Ok(packages)
}
