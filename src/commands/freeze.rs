/// Freeze command - generate requirements.txt from installed packages
use anyhow::Result;
use std::path::Path;
use std::fs;

pub async fn handle_freeze(output: Option<String>) -> Result<i32> {
    // Get installed packages
    let site_packages = crate::installer::SitePackages::default()?;
    let packages = site_packages.get_installed_packages()?;

    if packages.is_empty() {
        println!("No packages installed");
        return Ok(0);
    }

    // Format as requirements
    let mut requirements = Vec::new();
    for pkg_info in packages {
        // Parse package name and version from dist-info directory name
        // Format: package_name-version.dist-info
        if let Some(name_version) = pkg_info.strip_suffix(".dist-info") {
            if let Some(last_dash) = name_version.rfind('-') {
                let pkg_name = &name_version[..last_dash];
                let version = &name_version[last_dash + 1..];
                requirements.push(format!("{}=={}", pkg_name, version));
            }
        }
    }

    // Sort for consistency
    requirements.sort();

    // Output
    let output_text = requirements.join("\n");
    
    if let Some(output_file) = output {
        fs::write(&output_file, &output_text)?;
        println!("Wrote requirements to {}", output_file);
    } else {
        println!("{}", output_text);
    }

    Ok(0)
}
