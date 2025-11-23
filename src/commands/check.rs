/// Check command implementation
use crate::errors::PipError;
use crate::installer::SitePackages;
use crate::utils::color::get_color_output;

pub async fn handle_check(package: Option<String>) -> Result<i32, PipError> {
    if let Some(pkg) = package {
        check_package(&pkg).await
    } else {
        check_environment().await
    }
}

/// Check a specific package for issues
async fn check_package(package_name: &str) -> Result<i32, PipError> {
    let color = get_color_output();
    println!("Checking package: {}...", package_name);
    
    // Get site packages
    let site_packages = SitePackages::default().map_err(|e| PipError::FileSystemError {
        path: "site-packages".to_string(),
        operation: "access".to_string(),
        reason: e.to_string(),
    })?;
    
    // Check if package is installed
    let installed = site_packages.get_installed_packages().map_err(|e| PipError::FileSystemError {
        path: "site-packages".to_string(),
        operation: "list".to_string(),
        reason: e.to_string(),
    })?;
    
    if !installed.iter().any(|p| p.contains(package_name)) {
        color.print_error(&format!("Package '{}' is not installed", package_name));
        return Ok(1);
    }
    
    color.print_success(&format!("Package '{}' is installed", package_name));
    
    // Check for dist-info directory
    let dist_info_path = site_packages.path().join(format!("{}.dist-info", package_name));
    if dist_info_path.exists() {
        color.print_success(&format!("Metadata found at {}", dist_info_path.display()));
    } else {
        color.print_warning(&format!("Metadata not found for package '{}'", package_name));
    }
    
    Ok(0)
}

/// Check environment for issues
async fn check_environment() -> Result<i32, PipError> {
    let color = get_color_output();
    println!("Checking environment...\n");
    
    let mut issues = 0;
    
    // Check site packages
    color.print_header("Site Packages");
    match SitePackages::default() {
        Ok(site_packages) => {
            color.print_success(&format!("Site-packages location: {}", site_packages.path().display()));
            
            match site_packages.get_installed_packages() {
                Ok(packages) => {
                    color.print_success(&format!("Found {} installed packages", packages.len()));
                }
                Err(e) => {
                    color.print_error(&format!("Error listing packages: {}", e));
                    issues += 1;
                }
            }
        }
        Err(e) => {
            color.print_error(&format!("Error accessing site-packages: {}", e));
            issues += 1;
        }
    }
    
    // Check Python version
    color.print_header("Python Environment");
    if let Ok(version) = std::env::var("PYTHON_VERSION") {
        color.print_success(&format!("Python version: {}", version));
    } else {
        color.print_warning("Python version not set");
    }
    
    // Check virtual environment
    if std::env::var("VIRTUAL_ENV").is_ok() {
        color.print_success("Virtual environment: Active");
    } else {
        color.print_warning("Virtual environment: Not active");
    }
    
    // Check network connectivity
    color.print_header("Network");
    match std::net::ToSocketAddrs::to_socket_addrs(&("pypi.org", 443)) {
        Ok(_) => {
            color.print_success("PyPI connectivity: OK");
        }
        Err(e) => {
            color.print_error(&format!("PyPI connectivity: Failed - {}", e));
            issues += 1;
        }
    }
    
    // Summary
    color.print_header("Summary");
    if issues == 0 {
        color.print_success("No issues found");
        Ok(0)
    } else {
        color.print_error(&format!("Found {} issue(s)", issues));
        Ok(1)
    }
}
