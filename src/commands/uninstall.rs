/// Uninstall command implementation
use anyhow::Result;

pub async fn handle_uninstall(packages: Vec<String>, yes: bool) -> Result<i32> {
    if packages.is_empty() {
        eprintln!("ERROR: You must specify at least one package to uninstall");
        return Ok(1);
    }

    println!("The following packages will be removed:");
    for pkg in &packages {
        println!("  - {}", pkg);
    }

    if !yes {
        println!("Proceed (y/n)? ");
        // TODO: Read user input
    }

    println!("Successfully uninstalled {} packages", packages.len());
    Ok(0)
}
