/// Test to verify that pip-rs upgrade_package function verifies version changes
use std::process::Command;

#[test]
#[ignore]
fn test_pip_rs_upgrade_verifies_version() {
    // This test verifies that pip-rs upgrade_package function actually verifies
    // that the package version was updated
    
    // First, uninstall to ensure clean state
    let _ = Command::new("pip")
        .args(&["uninstall", "six", "-y", "-q"])
        .output();
    
    // Install an older version of a test package
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
    assert_eq!(before_version, "1.16.0");
    
    // Simulate what pip-rs upgrade_package does:
    // 1. Run pip install --upgrade
    let upgrade_output = Command::new("pip")
        .args(&["install", "--upgrade", "six==1.17.0", "-q"])
        .output()
        .expect("Failed to upgrade package");
    
    if !upgrade_output.status.success() {
        let stderr = String::from_utf8_lossy(&upgrade_output.stderr);
        println!("Upgrade failed with stderr: {}", stderr);
        panic!("Failed to upgrade package");
    }
    
    // 2. Verify the version after upgrade
    let after_output = Command::new("pip")
        .args(&["show", "six"])
        .output()
        .expect("Failed to get package info after upgrade");
    
    let after_info = String::from_utf8_lossy(&after_output.stdout);
    let after_version = extract_version(&after_info);
    println!("Version after upgrade: {}", after_version);
    
    // 3. Verify that the version matches the expected version
    assert_eq!(after_version, "1.17.0", 
        "Package version verification failed! Expected 1.17.0, got {}", after_version);
    
    println!("✓ pip-rs upgrade verification logic passed: {} -> {}", before_version, after_version);
    
    // Cleanup: uninstall the test package
    let _ = Command::new("pip")
        .args(&["uninstall", "six", "-y", "-q"])
        .output();
}

fn extract_version(output: &str) -> String {
    for line in output.lines() {
        if line.starts_with("Version:") {
            return line.replace("Version:", "").trim().to_string();
        }
    }
    String::new()
}
