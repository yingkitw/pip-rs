/// Search command implementation
use crate::errors::PipError;

pub async fn handle_search(query: &str) -> Result<i32, PipError> {
    println!("Searching for packages matching '{}'...", query);
    
    match crate::network::search_package(query).await {
        Ok(packages) => {
            if packages.is_empty() {
                println!("No packages found");
            } else {
                println!("Found {} packages:", packages.len());
                for pkg in packages {
                    println!("  {} - {}", pkg.name, pkg.summary.unwrap_or_default());
                }
            }
            Ok(0)
        }
        Err(e) => {
            Err(PipError::NetworkError {
                message: "Search failed".to_string(),
                retries: 0,
                last_error: e.to_string(),
            })
        }
    }
}
