/// Install command implementation
use crate::errors::PipError;
use crate::utils::progress;
use tempfile::TempDir;
use std::path::Path;
use pip_rs_core::{installer, models, config, resolver, network};

pub async fn handle_install(
    packages: Vec<String>,
    requirements: Option<String>,
    constraints: Option<String>,
    trusted_hosts: Vec<String>,
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

    // Parse constraints file if provided
    let mut constraint_reqs = Vec::new();
    if let Some(constraints_file) = constraints {
        let contents = std::fs::read_to_string(&constraints_file).map_err(|e| PipError::FileSystemError {
            path: constraints_file.clone(),
            operation: "read".to_string(),
            reason: e.to_string(),
        })?;
        for line in contents.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                match line.parse::<models::Requirement>() {
                    Ok(req) => constraint_reqs.push(req),
                    Err(e) => {
                        tracing::warn!("Invalid constraint: {} - {}", line, e);
                    }
                }
            }
        }
    }

    println!("Collecting packages...");

    // Parse requirements
    let mut parsed_reqs = Vec::new();
    for req_str in all_requirements {
        match req_str.parse::<models::Requirement>() {
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

    // Load config and merge trusted hosts
    let mut config = config::config::Config::new();
    for host in trusted_hosts {
        config.add_trusted_host(host);
    }
    
    // Smart defaults: Auto-detect venv
    let venv_path = std::env::var("VIRTUAL_ENV").ok();
    if let Some(ref venv) = venv_path {
        tracing::debug!("Detected virtual environment: {}", venv);
    }

    // Resolve dependencies
    println!("\nResolving dependencies...");
    let mut resolver = resolver::Resolver::new();
    if !constraint_reqs.is_empty() {
        resolver.set_constraints(constraint_reqs);
    }
    let resolved = resolver.resolve(parsed_reqs).await.map_err(|e| PipError::InstallationFailed {
        package: "dependencies".to_string(),
        reason: e.to_string(),
    })?;

    println!("Successfully resolved {} packages:", resolved.len());
    for pkg in &resolved {
        println!("  - {} {}", pkg.name, pkg.version);
    }

    // Download and install packages
    let temp_dir = TempDir::new().map_err(|e| PipError::FileSystemError {
        path: "temp".to_string(),
        operation: "create directory".to_string(),
        reason: e.to_string(),
    })?;
    
    let total = resolved.len();
    let pb = if progress::is_quiet() {
        None
    } else {
        Some(progress::progress_bar(total as u64, "Installing"))
    };
    
    let mut installed_count = 0;
    let mut failed_count = 0;

    for pkg in &resolved {
        if let Some(prog) = &pb {
            prog.set_message(format!("{} {}", pkg.name, pkg.version));
        }
        
        match install_package(pkg, temp_dir.path()).await {
            Ok(_) => {
                installed_count += 1;
            }
            Err(e) => {
                if !progress::is_quiet() {
                    eprintln!("✗ Failed to install {} {}: {}", pkg.name, pkg.version, e);
                }
                failed_count += 1;
            }
        }
        
        if let Some(prog) = &pb {
            prog.inc(1);
        }
    }

    if let Some(pb) = pb {
        if failed_count > 0 {
            progress::finish_error(&pb, &format!("Installed {} packages ({} failed)", installed_count, failed_count));
        } else {
            progress::finish_success(&pb, &format!("Installed {} packages", installed_count));
        }
    }

    if failed_count > 0 {
        return Ok(1);
    }

    Ok(0)
}

/// Install a single package by downloading and extracting its wheel
async fn install_package(pkg: &models::Package, temp_dir: &Path) -> Result<(), PipError> {
    // Find wheel URL
    let wheel_url = network::find_wheel_url(&pkg.name, &pkg.version)
        .await
        .map_err(|e| PipError::PackageNotFound {
            name: pkg.name.clone(),
            version: Some(pkg.version.clone()),
        })?;
    
    // Download wheel
    // eprintln!("  Downloading {} from {}", pkg.name, wheel_url);
    let wheel_data = network::PackageClient::new()
        .download_package(&wheel_url)
        .await
        .map_err(|e| PipError::NetworkError {
            message: format!("Failed to download {}", pkg.name),
            retries: 0,
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
    let wheel = installer::wheel::WheelFile::new(wheel_path).map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    let site_packages = installer::SitePackages::default().map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    let installer = installer::PackageInstaller::new(site_packages);
    installer.install_wheel(&wheel).await.map_err(|e| PipError::InstallationFailed {
        package: pkg.name.clone(),
        reason: e.to_string(),
    })?;
    
    Ok(())
}
