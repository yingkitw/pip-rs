/// PyPI API interactions
use crate::models::Package;
use anyhow::{Result, anyhow};

pub async fn search_package(query: &str) -> Result<Vec<Package>> {
    // TODO: Implement PyPI search API
    // This would call the PyPI JSON API
    let _ = query;
    Ok(Vec::new())
}

/// Find the best wheel URL for a package version
pub async fn find_wheel_url(package_name: &str, version: &str) -> Result<String> {
    let info = super::GLOBAL_CLIENT.get_package_info(package_name).await?;
    
    let urls = info["urls"]
        .as_array()
        .ok_or_else(|| anyhow!("No URLs found for package"))?;
    
    // Prefer pure Python wheels (py3-none-any), then platform-specific
    for url_info in urls {
        let filename = url_info["filename"]
            .as_str()
            .ok_or_else(|| anyhow!("No filename in URL info"))?;
        
        // Look for .whl files
        if filename.ends_with(".whl") {
            // Prefer pure Python wheels
            if filename.contains("py3-none-any") {
                let url = url_info["url"]
                    .as_str()
                    .ok_or_else(|| anyhow!("No URL in URL info"))?;
                return Ok(url.to_string());
            }
        }
    }
    
    // If no pure Python wheel found, get any wheel
    for url_info in urls {
        let filename = url_info["filename"]
            .as_str()
            .ok_or_else(|| anyhow!("No filename in URL info"))?;
        
        if filename.ends_with(".whl") {
            let url = url_info["url"]
                .as_str()
                .ok_or_else(|| anyhow!("No URL in URL info"))?;
            return Ok(url.to_string());
        }
    }
    
    Err(anyhow!("No wheel found for {} {}", package_name, version))
}

#[allow(dead_code)]
pub async fn get_package_releases(package_name: &str) -> Result<Vec<String>> {
    let info = super::GLOBAL_CLIENT.get_package_info(package_name).await?;
    
    let releases = info["releases"]
        .as_object()
        .map(|r| r.keys().cloned().collect())
        .unwrap_or_default();
    
    Ok(releases)
}

#[allow(dead_code)]
pub async fn get_latest_version(package_name: &str) -> Result<String> {
    let info = super::GLOBAL_CLIENT.get_package_info(package_name).await?;
    
    let version = info["info"]["version"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No version found"))?
        .to_string();
    
    Ok(version)
}

pub async fn get_package_metadata(package_name: &str, version: &str) -> Result<Package> {
    // TODO: Implement disk caching
    // For now, fetch directly from PyPI
    
    let info = super::GLOBAL_CLIENT.get_package_info(package_name).await?;
    
    let pkg_info = &info["info"];
    
    let package = Package {
        name: pkg_info["name"].as_str().unwrap_or(package_name).to_string(),
        version: pkg_info["version"].as_str().unwrap_or(version).to_string(),
        summary: pkg_info["summary"].as_str().map(|s| s.to_string()),
        home_page: pkg_info["home_page"].as_str().map(|s| s.to_string()),
        author: pkg_info["author"].as_str().map(|s| s.to_string()),
        license: pkg_info["license"].as_str().map(|s| s.to_string()),
        requires_python: pkg_info["requires_python"].as_str().map(|s| s.to_string()),
        requires_dist: pkg_info["requires_dist"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        classifiers: pkg_info["classifiers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    };
    
    // TODO: Cache the package for future requests
    
    Ok(package)
}
