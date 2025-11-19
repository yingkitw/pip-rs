/// Debug command - display system and environment information
use anyhow::Result;

pub async fn handle_debug() -> Result<i32> {
    println!("pip-rs debug information\n");

    // System information
    println!("=== System Information ===");
    print_system_info();

    // Python information
    println!("\n=== Python Information ===");
    print_python_info();

    // pip-rs configuration
    println!("\n=== pip-rs Configuration ===");
    print_pip_config();

    // Installed packages
    println!("\n=== Installed Packages ===");
    print_installed_packages().await?;

    // Network information
    println!("\n=== Network Information ===");
    print_network_info();

    Ok(0)
}

fn print_system_info() {
    println!("OS: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!("Family: {}", std::env::consts::FAMILY);

    #[cfg(target_os = "macos")]
    {
        println!("Platform: macOS");
    }
    #[cfg(target_os = "linux")]
    {
        println!("Platform: Linux");
    }
    #[cfg(target_os = "windows")]
    {
        println!("Platform: Windows");
    }

    if let Ok(home) = std::env::var("HOME") {
        println!("Home: {}", home);
    }

    if let Ok(path) = std::env::var("PATH") {
        let paths: Vec<&str> = path.split(':').collect();
        println!("PATH entries: {}", paths.len());
    }
}

fn print_python_info() {
    // Get Python version from environment
    if let Ok(version) = std::env::var("PYTHON_VERSION") {
        println!("Python version: {}", version);
    } else {
        println!("Python version: Not set");
    }

    // Get site-packages location
    match crate::installer::SitePackages::default() {
        Ok(site_packages) => {
            println!("Site-packages: {}", site_packages.path().display());
        }
        Err(e) => {
            println!("Site-packages: Error - {}", e);
        }
    }

    // Virtual environment detection
    if std::env::var("VIRTUAL_ENV").is_ok() {
        println!("Virtual environment: Active");
    } else {
        println!("Virtual environment: Not active");
    }
}

fn print_pip_config() {
    println!("pip-rs version: 0.1.0");
    println!("Rust edition: 2021");

    // Check for pip.conf
    let home = std::env::var("HOME").unwrap_or_default();
    let pip_conf = format!("{}/.pip/pip.conf", home);
    if std::path::Path::new(&pip_conf).exists() {
        println!("pip.conf: Found at {}", pip_conf);
    } else {
        println!("pip.conf: Not found");
    }

    // Check for pyproject.toml
    if std::path::Path::new("pyproject.toml").exists() {
        println!("pyproject.toml: Found");
    } else {
        println!("pyproject.toml: Not found");
    }

    // Check for requirements.txt
    if std::path::Path::new("requirements.txt").exists() {
        println!("requirements.txt: Found");
    } else {
        println!("requirements.txt: Not found");
    }
}

async fn print_installed_packages() -> Result<()> {
    match crate::installer::SitePackages::default() {
        Ok(site_packages) => {
            match site_packages.get_installed_packages() {
                Ok(packages) => {
                    println!("Total packages: {}", packages.len());
                    if packages.len() <= 10 {
                        for pkg in packages {
                            println!("  - {}", pkg);
                        }
                    } else {
                        for pkg in packages.iter().take(5) {
                            println!("  - {}", pkg);
                        }
                        println!("  ... and {} more", packages.len() - 5);
                    }
                }
                Err(e) => {
                    println!("Error listing packages: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error accessing site-packages: {}", e);
        }
    }

    Ok(())
}

fn print_network_info() {
    // Check network connectivity
    println!("PyPI URL: https://pypi.org/simple/");
    println!("Network: Checking connectivity...");

    // Try to resolve PyPI
    match std::net::ToSocketAddrs::to_socket_addrs(&("pypi.org", 443)) {
        Ok(addrs) => {
            let count = addrs.count();
            println!("PyPI resolution: OK ({} addresses)", count);
        }
        Err(e) => {
            println!("PyPI resolution: Failed - {}", e);
        }
    }

    // Check for proxy settings
    if let Ok(proxy) = std::env::var("HTTP_PROXY") {
        println!("HTTP_PROXY: {}", proxy);
    }
    if let Ok(proxy) = std::env::var("HTTPS_PROXY") {
        println!("HTTPS_PROXY: {}", proxy);
    }

    // Check for pip index settings
    if let Ok(index) = std::env::var("PIP_INDEX_URL") {
        println!("PIP_INDEX_URL: {}", index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_command() {
        let result = handle_debug().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
