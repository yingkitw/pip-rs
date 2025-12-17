/// Download command - download packages without installing
use crate::errors::PipError;
use anyhow::{Result, anyhow};
use std::path::Path;
use pip_rs_core::{models, resolver, network};

pub async fn handle_download(
    packages: Vec<String>,
    requirements: Option<String>,
    destination: Option<String>,
) -> Result<i32, PipError> {
    if packages.is_empty() && requirements.is_none() {
        return Err(PipError::InvalidRequirement {
            spec: "None".to_string(),
            reason: "You must give at least one requirement to download".to_string(),
        });
    }

    let mut all_requirements = Vec::new();

    // Parse package arguments
    for pkg in packages {
        all_requirements.push(pkg);
    }

    // Parse requirements file if provided
    if let Some(req_file) = requirements {
        let contents = std::fs::read_to_string(&req_file).map_err(|e| PipError::FileSystemError {
            path: req_file.clone(),
            operation: "read".to_string(),
            reason: e.to_string(),
        })?;
        for line in contents.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                all_requirements.push(line.to_string());
            }
        }
    }

    println!("Collecting packages...");

    // Parse requirements
    let mut parsed_reqs = Vec::new();
    for req_str in all_requirements {
        match req_str.parse::<models::Requirement>() {
            Ok(req) => {
                println!("  - {}", req.name);
                parsed_reqs.push(req);
            }
            Err(e) => {
                return Err(PipError::InvalidRequirement {
                    spec: req_str,
                    reason: e.to_string(),
                });
            }
        }
    }

    if parsed_reqs.is_empty() {
        return Err(PipError::InvalidRequirement {
            spec: "None".to_string(),
            reason: "No valid requirements found".to_string(),
        });
    }

    // Resolve dependencies
    println!("\nResolving dependencies...");
    let mut resolver = resolver::Resolver::new();
    let resolved = resolver.resolve(parsed_reqs).await.map_err(|e| PipError::InstallationFailed {
        package: "dependencies".to_string(),
        reason: e.to_string(),
    })?;

    println!("Successfully resolved {} packages:", resolved.len());
    for pkg in &resolved {
        println!("  - {} {}", pkg.name, pkg.version);
    }

    // Determine destination directory
    let dest_dir = destination.unwrap_or_else(|| ".".to_string());
    let dest_path = Path::new(&dest_dir);
    
    if !dest_path.exists() {
        std::fs::create_dir_all(dest_path).map_err(|e| PipError::FileSystemError {
            path: dest_dir.clone(),
            operation: "create_dir".to_string(),
            reason: e.to_string(),
        })?;
    }

    // Download packages
    println!("\nDownloading packages to {}...", dest_dir);
    
    let mut downloaded_count = 0;
    let mut failed_count = 0;

    for pkg in &resolved {
        match download_package(pkg, dest_path).await {
            Ok(filename) => {
                println!("✓ Downloaded {} to {}", pkg.name, filename);
                downloaded_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to download {} {}: {}", pkg.name, pkg.version, e);
                failed_count += 1;
            }
        }
    }

    println!("\nDownload complete!");
    println!("  Successfully downloaded: {}", downloaded_count);
    if failed_count > 0 {
        println!("  Failed: {}", failed_count);
        return Ok(1);
    }

    Ok(0)
}

/// Download a single package wheel
async fn download_package(pkg: &models::Package, dest_dir: &Path) -> Result<String> {
    // Find wheel URL
    let wheel_url = network::find_wheel_url(&pkg.name, &pkg.version).await?;
    
    // Download wheel
    eprintln!("  Downloading {} from {}", pkg.name, wheel_url);
    let wheel_data = network::PackageClient::new().download_package(&wheel_url).await?;
    
    // Extract filename from URL
    let filename = wheel_url
        .split('/')
        .last()
        .ok_or_else(|| anyhow!("Invalid wheel URL"))?;
    
    // Save wheel to destination
    let wheel_path = dest_dir.join(filename);
    std::fs::write(&wheel_path, wheel_data)?;
    
    Ok(filename.to_string())
}
