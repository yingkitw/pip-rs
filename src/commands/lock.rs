/// Lock command - generate lock files for reproducible installs
use crate::errors::PipError;
use anyhow::Result;
use std::path::Path;
use pip_rs_core::{models, resolver, network, installer};

pub async fn handle_lock(
    requirements: Option<String>,
    output: Option<String>,
) -> Result<i32, PipError> {
    if requirements.is_none() {
        eprintln!("ERROR: You must provide a requirements file with -r/--requirements");
        return Ok(1);
    }

    let req_file = requirements.unwrap();
    if !Path::new(&req_file).exists() {
        eprintln!("ERROR: Requirements file not found: {}", req_file);
        return Ok(1);
    }

    println!("Reading requirements from {}...", req_file);

    // Parse requirements file
    let contents = std::fs::read_to_string(&req_file).map_err(|e| PipError::FileSystemError {
        path: req_file.clone(),
        operation: "read".to_string(),
        reason: e.to_string(),
    })?;
    let mut all_requirements = Vec::new();

    for line in contents.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            all_requirements.push(line.to_string());
        }
    }

    if all_requirements.is_empty() {
        eprintln!("ERROR: No requirements found in {}", req_file);
        return Ok(1);
    }

    println!("Parsing {} requirements...", all_requirements.len());

    // Parse requirements
    let mut parsed_reqs = Vec::new();
    for req_str in all_requirements {
        match req_str.parse::<models::Requirement>() {
            Ok(req) => {
                println!("  - {}", req.name);
                parsed_reqs.push(req);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse requirement '{}': {}", req_str, e);
            }
        }
    }

    if parsed_reqs.is_empty() {
        eprintln!("ERROR: No valid requirements found");
        return Ok(1);
    }

    // Resolve dependencies
    println!("\nResolving dependencies...");
    let mut resolver = resolver::Resolver::new();
    let resolved = resolver.resolve(parsed_reqs).await.map_err(|e| PipError::DependencyResolutionError {
        package: "requirements".to_string(),
        reason: e.to_string(),
    })?;

    println!("Successfully resolved {} packages:", resolved.len());
    for pkg in &resolved {
        println!("  - {} {}", pkg.name, pkg.version);
    }

    // Create lock file
    println!("\nGenerating lock file...");
    let python_version = format!("{}.{}", 3, 11); // Default to 3.11
    let lockfile = resolver::LockFile::from_packages(resolved, python_version);

    // Validate lock file
    lockfile.validate().map_err(|e| PipError::InvalidPackage {
        name: "lockfile".to_string(),
        reason: e.to_string(),
    })?;

    // Save lock file
    let lock_path = output.unwrap_or_else(|| "pip-lock.json".to_string());
    lockfile.save(Path::new(&lock_path)).map_err(|e| PipError::FileSystemError {
        path: lock_path.clone(),
        operation: "save".to_string(),
        reason: e.to_string(),
    })?;

    println!("\n✓ Lock file generated: {}", lock_path);
    println!("  Packages: {}", lockfile.packages.len());
    println!("  Python version: {}", lockfile.python_version);
    println!("  Generated at: {}", lockfile.generated_at);

    Ok(0)
}

#[allow(dead_code)]
pub async fn handle_lock_install(
    lock_file: String,
) -> Result<i32, PipError> {
    if !Path::new(&lock_file).exists() {
        eprintln!("ERROR: Lock file not found: {}", lock_file);
        return Ok(1);
    }

    println!("Reading lock file: {}", lock_file);

    // Load lock file
    let lockfile = resolver::LockFile::load(Path::new(&lock_file)).map_err(|e| PipError::FileSystemError {
        path: lock_file.clone(),
        operation: "load".to_string(),
        reason: e.to_string(),
    })?;

    // Validate lock file
    lockfile.validate().map_err(|e| PipError::InvalidPackage {
        name: "lockfile".to_string(),
        reason: e.to_string(),
    })?;

    println!("Lock file validated");
    println!("  Packages: {}", lockfile.packages.len());
    println!("  Python version: {}", lockfile.python_version);
    println!("  Generated at: {}", lockfile.generated_at);

    // Convert to packages
    let packages = lockfile.to_packages();

    // Install packages
    println!("\nInstalling {} packages from lock file...", packages.len());
    
    let temp_dir = tempfile::TempDir::new().map_err(|e| PipError::FileSystemError {
        path: "temp".to_string(),
        operation: "create".to_string(),
        reason: e.to_string(),
    })?;
    let mut installed_count = 0;
    let mut failed_count = 0;

    for pkg in &packages {
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
    
    // Cleanup happens automatically when TempDir is dropped
    
    println!("\nInstallation complete!");
    println!("  Successfully installed: {}", installed_count);
    if failed_count > 0 {
        println!("  Failed: {}", failed_count);
        return Ok(1);
    }

    Ok(0)
}

/// Install a single package by downloading and extracting its wheel
#[allow(dead_code)]
async fn install_package(pkg: &models::Package, temp_dir: &std::path::Path) -> Result<(), PipError> {
    // Find wheel URL
    let wheel_url = network::find_wheel_url(&pkg.name, &pkg.version).await.map_err(|_e| PipError::PackageNotFound {
        name: pkg.name.clone(),
        version: Some(pkg.version.clone()),
    })?;
    
    // Download wheel
    eprintln!("  Downloading {} from {}", pkg.name, wheel_url);
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
    let wheel = installer::wheel::WheelFile::new(wheel_path).map_err(|e| PipError::InvalidPackage {
        name: pkg.name.clone(),
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
