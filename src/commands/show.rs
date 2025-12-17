/// Show command implementation
use crate::errors::PipError;
use pip_rs_core::installer::site_packages::SitePackages;

pub async fn handle_show(package: &str) -> Result<i32, PipError> {
    println!("Fetching information for package: {}", package);
    
    // Get site packages
    let site_packages = SitePackages::default().map_err(|e| PipError::FileSystemError {
        path: "site-packages".to_string(),
        operation: "access".to_string(),
        reason: e.to_string(),
    })?;
    
    match site_packages.get_package_details(package) {
        Ok(Some(info)) => {
            println!("Name: {}", info.name);
            println!("Version: {}", info.version);
            println!("Location: {}", info.location.display());
            if !info.requires.is_empty() {
                println!("Requires: {}", info.requires.join(", "));
            }
            Ok(0)
        }
        Ok(None) => {
            eprintln!("Package '{}' not found", package);
            Ok(1)
        }
        Err(e) => {
            eprintln!("Error getting package info for '{}': {}", package, e);
            Ok(1)
        }
    }
}
