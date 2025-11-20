/// Uninstall command implementation
use crate::errors::PipError;
use std::io::{self, BufRead};

pub async fn handle_uninstall(packages: Vec<String>, yes: bool) -> Result<i32, PipError> {
    if packages.is_empty() {
        return Err(PipError::InvalidRequirement {
            spec: "None".to_string(),
            reason: "You must specify at least one package to uninstall".to_string(),
        });
    }

    println!("The following packages will be removed:");
    for pkg in &packages {
        println!("  - {}", pkg);
    }

    // Get confirmation if not --yes flag
    if !yes {
        println!("\nProceed (y/n)? ");
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).map_err(|e| PipError::FileSystemError {
            path: "stdin".to_string(),
            operation: "read".to_string(),
            reason: e.to_string(),
        })?;
        
        let response = line.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("Aborted");
            return Ok(0);
        }
    }

    // Uninstall packages
    let site_packages = crate::installer::SitePackages::default().map_err(|e| PipError::InstallationFailed {
        package: "site-packages".to_string(),
        reason: e.to_string(),
    })?;
    let installer = crate::installer::PackageInstaller::new(site_packages);
    
    let mut uninstalled_count = 0;
    let mut failed_count = 0;

    for pkg_name in packages {
        match installer.uninstall(&pkg_name).await {
            Ok(_) => {
                println!("✓ Successfully uninstalled {}", pkg_name);
                uninstalled_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to uninstall {}: {}", pkg_name, e);
                failed_count += 1;
            }
        }
    }

    println!("\nUninstall complete!");
    println!("  Successfully uninstalled: {}", uninstalled_count);
    if failed_count > 0 {
        println!("  Failed: {}", failed_count);
        return Ok(1);
    }

    Ok(0)
}
