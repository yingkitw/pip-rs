/// Package installation/upgrade functionality
use std::process::Command;
use super::traits::UpgradeResult;

/// Get the installed version of a package
fn get_installed_version(name: &str) -> Option<String> {
    let output = Command::new("pip")
        .args(&["show", name])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let info = String::from_utf8_lossy(&output.stdout);
    for line in info.lines() {
        if line.starts_with("Version:") {
            return Some(line.replace("Version:", "").trim().to_string());
        }
    }
    None
}

pub fn upgrade_package(name: &str, current: &str, latest: &str) -> UpgradeResult {
    let package_spec = format!("{}=={}", name, latest);
    
    let output = Command::new("pip")
        .args(&["install", "--upgrade", &package_spec, "-q"])
        .output();
    
    match output {
        Ok(result) => {
            if !result.status.success() {
                return UpgradeResult {
                    name: name.to_string(),
                    current_version: current.to_string(),
                    latest_version: latest.to_string(),
                    success: false,
                    error_msg: Some("Installation failed".to_string()),
                };
            }
            
            // Verify that the package was actually upgraded by checking the installed version
            match get_installed_version(name) {
                Some(installed_version) => {
                    let success = installed_version == latest;
                    UpgradeResult {
                        name: name.to_string(),
                        current_version: current.to_string(),
                        latest_version: latest.to_string(),
                        success,
                        error_msg: if success {
                            None
                        } else {
                            Some(format!(
                                "Version mismatch: expected {}, got {}",
                                latest, installed_version
                            ))
                        },
                    }
                }
                None => {
                    UpgradeResult {
                        name: name.to_string(),
                        current_version: current.to_string(),
                        latest_version: latest.to_string(),
                        success: false,
                        error_msg: Some("Could not verify installed version".to_string()),
                    }
                }
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

/// Upgrade multiple packages in parallel with bounded concurrency
pub async fn upgrade_packages_parallel(
    packages: Vec<(String, String, String)>,
    concurrency: usize,
) -> Vec<UpgradeResult> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use futures::future::join_all;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = vec![];

    for (name, current, latest) in packages {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.ok();
            upgrade_package(&name, &current, &latest)
        });
        handles.push(handle);
    }

    let results = join_all(handles).await;
    results.into_iter().filter_map(|r| r.ok()).collect()
}
