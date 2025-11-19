/// Package installation/upgrade functionality
use std::process::Command;

pub struct UpgradeResult {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub success: bool,
    pub error_msg: Option<String>,
}

pub fn upgrade_package(name: &str, current: &str, latest: &str) -> UpgradeResult {
    let package_spec = format!("{}=={}", name, latest);
    
    let output = Command::new("pip")
        .args(&["install", "--upgrade", &package_spec, "-q"])
        .output();
    
    match output {
        Ok(result) => {
            UpgradeResult {
                name: name.to_string(),
                current_version: current.to_string(),
                latest_version: latest.to_string(),
                success: result.status.success(),
                error_msg: if result.status.success() {
                    None
                } else {
                    Some("Installation failed".to_string())
                },
            }
        }
        Err(e) => {
            UpgradeResult {
                name: name.to_string(),
                current_version: current.to_string(),
                latest_version: latest.to_string(),
                success: false,
                error_msg: Some(e.to_string()),
            }
        }
    }
}
