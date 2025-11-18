/// Site-packages management
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

pub struct SitePackages {
    path: PathBuf,
}

impl SitePackages {
    pub fn new(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Get the default site-packages directory
    pub fn default() -> Result<Self> {
        let path = if cfg!(target_os = "windows") {
            PathBuf::from("Lib/site-packages")
        } else {
            PathBuf::from("lib/python3.11/site-packages")
        };
        Self::new(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn install_file(&self, source: &Path, relative_path: &Path) -> Result<PathBuf> {
        let target = self.path.join(relative_path);
        
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::copy(source, &target)?;
        Ok(target)
    }

    pub fn install_directory(&self, source: &Path, relative_path: &Path) -> Result<()> {
        let target = self.path.join(relative_path);
        fs::create_dir_all(&target)?;
        
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let target_path = target.join(&file_name);
            
            if path.is_dir() {
                self.install_directory(&path, &relative_path.join(&file_name))?;
            } else {
                fs::copy(&path, &target_path)?;
            }
        }
        
        Ok(())
    }

    pub fn is_installed(&self, package_name: &str) -> bool {
        let dist_info = self.path.join(format!("{}.dist-info", package_name));
        dist_info.exists()
    }

    pub fn get_installed_packages(&self) -> Result<Vec<String>> {
        let mut packages = Vec::new();
        
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.ends_with(".dist-info") {
                            let pkg_name = name_str.trim_end_matches(".dist-info").to_string();
                            packages.push(pkg_name);
                        }
                    }
                }
            }
        }
        
        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_site_packages_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let site_packages = SitePackages::new(temp_dir.path().to_path_buf())?;
        assert!(site_packages.path().exists());
        Ok(())
    }

    #[test]
    fn test_is_installed() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let site_packages = SitePackages::new(temp_dir.path().to_path_buf())?;
        
        // Create a dist-info directory
        fs::create_dir_all(site_packages.path().join("requests.dist-info"))?;
        
        assert!(site_packages.is_installed("requests"));
        assert!(!site_packages.is_installed("numpy"));
        Ok(())
    }
}
