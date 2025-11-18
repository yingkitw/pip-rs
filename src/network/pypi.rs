/// PyPI API interactions
use crate::models::Package;
use anyhow::Result;

pub async fn search_package(query: &str) -> Result<Vec<Package>> {
    // TODO: Implement PyPI search API
    // This would call the PyPI JSON API
    let _ = query;
    Ok(Vec::new())
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
    
    Ok(package)
}
