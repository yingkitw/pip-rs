/// Show command implementation
use crate::errors::PipError;

pub async fn handle_show(package: &str) -> Result<i32, PipError> {
    println!("Fetching information for package: {}", package);
    
    match crate::network::get_package_metadata(package, "latest").await {
        Ok(pkg) => {
            println!("Name: {}", pkg.name);
            println!("Version: {}", pkg.version);
            if let Some(summary) = pkg.summary {
                println!("Summary: {}", summary);
            }
            if let Some(home_page) = pkg.home_page {
                println!("Home-page: {}", home_page);
            }
            if let Some(author) = pkg.author {
                println!("Author: {}", author);
            }
            if let Some(license) = pkg.license {
                println!("License: {}", license);
            }
            if !pkg.requires_dist.is_empty() {
                println!("Requires:");
                for req in pkg.requires_dist {
                    println!("  - {}", req);
                }
            }
            Ok(0)
        }
        Err(e) => {
            Err(PipError::PackageNotFound {
                name: package.to_string(),
                version: None,
            })
        }
    }
}
