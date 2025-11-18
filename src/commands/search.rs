/// Search command implementation
use anyhow::Result;

pub async fn handle_search(query: &str) -> Result<i32> {
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
            eprintln!("ERROR: Search failed: {}", e);
            Ok(1)
        }
    }
}
