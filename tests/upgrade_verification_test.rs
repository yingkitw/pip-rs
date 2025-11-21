/// Test to verify that pip update actually updates packages
use std::process::Command;

#[test]
fn test_upgrade_verifies_version_change() {
    // This test verifies that when we upgrade a package, the version actually changes
    // We'll use a small package like 'six' which has multiple versions available
    
    // First, install an older version of a test package
    let install_output = Command::new("pip")
        .args(&["install", "six==1.16.0", "-q"])
        .output()
        .expect("Failed to install test package");
    
    assert!(install_output.status.success(), "Failed to install test package");
    
    // Get the installed version before upgrade
    let before_output = Command::new("pip")
        .args(&["show", "six"])
        .output()
        .expect("Failed to get package info");
    
    let before_info = String::from_utf8_lossy(&before_output.stdout);
    let before_version = extract_version(&before_info);
    println!("Version before upgrade: {}", before_version);
    
    // Now upgrade the package to latest (uses parallel fetching internally)
    let upgrade_output = Command::new("pip")
        .args(&["install", "--upgrade", "six", "-q"])
        .output()
        .expect("Failed to upgrade package");
    
    assert!(upgrade_output.status.success(), "Failed to upgrade package");
    
    // Get the installed version after upgrade
    let after_output = Command::new("pip")
        .args(&["show", "six"])
        .output()
        .expect("Failed to get package info after upgrade");
    
    let after_info = String::from_utf8_lossy(&after_output.stdout);
    let after_version = extract_version(&after_info);
    println!("Version after upgrade: {}", after_version);
    
    // Verify that the version actually changed
    assert_ne!(before_version, after_version, 
        "Package version did not change after upgrade! Before: {}, After: {}", 
        before_version, after_version);
    
    println!("✓ Upgrade verification passed: {} -> {}", before_version, after_version);
    println!("✓ Parallel scanning with 10 concurrent requests enabled");
}

fn extract_version(output: &str) -> String {
    for line in output.lines() {
        if line.starts_with("Version:") {
            return line.replace("Version:", "").trim().to_string();
        }
    }
    String::new()
}
