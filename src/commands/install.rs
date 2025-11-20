/// Install command implementation
use crate::errors::PipError;
use tempfile::TempDir;
use std::path::Path;

pub async fn handle_install(
    packages: Vec<String>,
    requirements: Option<String>,
    _target: Option<String>,
) -> Result<i32, PipError> {
    if packages.is_empty() && requirements.is_none() {
        return Err(PipError::InvalidRequirement {
            spec: "None".to_string(),
            reason: "You must give at least one requirement to install".to_string(),
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
        match req_str.parse::<crate::models::Requirement>() {
            Ok(req) => {
                if req.extras.is_empty() {
                    println!("  - {}", req.name);
                } else {
                    println!("  - {}[{}]", req.name, req.extras.join(","));
                }
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
    let mut resolver = crate::resolver::Resolver::new();
    let resolved = resolver.resolve(parsed_reqs).await.map_err(|e| PipError::InstallationFailed {
        package: "dependencies".to_string(),
        reason: e.to_string(),
    })?;

    println!("Successfully resolved {} packages:", resolved.len());
    for pkg in &resolved {
        println!("  - {} {}", pkg.name, pkg.version);
    }

    // Download and install packages
    println!("\nDownloading and installing packages...");
    
    let temp_dir = TempDir::new().map_err(|e| PipError::FileSystemError {
        path: "temp".to_string(),
        operation: "create directory".to_string(),
        reason: e.to_string(),
    })?;
    let mut installed_count = 0;
    let mut failed_count = 0;

    for pkg in &resolved {
        match install_package(pkg, temp_dir.path()).await {
            Ok(_) => {
                println!("✓ Successfully installed {} {}", pkg.name, pkg.version);
                installed_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to install {} {}: {}", pkg.name, pkg.version, e);
                failed_count += 1;
            }
        }
    }

    println!("\nInstallation complete!");
    println!("  Successfully installed: {}", installed_count);
    if failed_count > 0 {
        println!("  Failed: {}", failed_count);
        return Ok(1);
    }

    Ok(0)
}

/// Install a single package by downloading and extracting its wheel
async fn install_package(pkg: &crate::models::Package, temp_dir: &Path) -> Result<(), PipError> {
    // Find wheel URL
    let wheel_url = crate::network::find_wheel_url(&pkg.name, &pkg.version)
        .await
        .map_err(|e| PipError::PackageNotFound {
            name: pkg.name.clone(),
            version: Some(pkg.version.clone()),
        })?;
    
    // Download wheel
    eprintln!("  Downloading {} from {}", pkg.name, wheel_url);
    let wheel_data = crate::network::PackageClient::new()
        .download_package(&wheel_url)
        .await
        .map_err(|e| PipError::NetworkError {
            message: format!("Failed to download {}", wheel_url),
            retries: 3, // Placeholder
            last_error: e.to_string(),
        })?;
    
    // Save wheel to temp directory
    let wheel_filename = format!("{}-{}.whl", pkg.name, pkg.version);
    let wheel_path = temp_dir.join(&wheel_filename);
    std::fs::write(&wheel_path, wheel_data).map_err(|e| PipError::FileSystemError {
        path: wheel_path.to_string_lossy().to_string(),
        operation: "write".to_string(),
        reason: e.to_string(),
    })?;
    
    // Extract and install wheel
    let wheel = crate::installer::wheel::WheelFile::new(wheel_path).map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    let site_packages = crate::installer::SitePackages::default().map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    let installer = crate::installer::PackageInstaller::new(site_packages);
    installer.install_wheel(&wheel).await.map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    
    Ok(())
}
