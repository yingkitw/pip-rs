/// Package installation/upgrade functionality
use std::process::Command;
use super::traits::UpgradeResult;

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
