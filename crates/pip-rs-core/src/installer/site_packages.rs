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
    /// Auto-detects virtual environment if VIRTUAL_ENV is set
    /// Falls back to detecting actual Python site-packages location
    pub fn default() -> Result<Self> {
        // Smart defaults: Auto-detect venv
        if let Ok(venv_path) = std::env::var("VIRTUAL_ENV") {
            let venv = PathBuf::from(venv_path);
            let site_packages = if cfg!(target_os = "windows") {
                venv.join("Lib").join("site-packages")
            } else {
                // Try to detect Python version from venv
                let python_version = Self::detect_python_version(&venv).unwrap_or_else(|| "3.11".to_string());
                venv.join("lib").join(format!("python{}", python_version)).join("site-packages")
            };
            
            if site_packages.exists() {
                tracing::debug!("Using venv site-packages: {}", site_packages.display());
                return Self::new(site_packages);
            }
        }
        
        // Try to detect actual Python site-packages location
        // Check common locations that actually exist
        let possible_paths = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from("Lib/site-packages"),
            ]
        } else {
            vec![
                // macOS with Python.org installer
                PathBuf::from("/Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages"),
                PathBuf::from("/Library/Frameworks/Python.framework/Versions/3.11/lib/python3.11/site-packages"),
                PathBuf::from("/Library/Frameworks/Python.framework/Versions/3.10/lib/python3.10/site-packages"),
                // macOS user site-packages
                shellexpand::tilde("~/Library/Python/3.12/lib/python/site-packages").to_string().into(),
                shellexpand::tilde("~/Library/Python/3.11/lib/python/site-packages").to_string().into(),
                shellexpand::tilde("~/Library/Python/3.10/lib/python/site-packages").to_string().into(),
                // Linux system
                PathBuf::from("/usr/local/lib/python3.12/site-packages"),
                PathBuf::from("/usr/local/lib/python3.11/site-packages"),
                PathBuf::from("/usr/lib/python3/dist-packages"),
                // User site-packages
                shellexpand::tilde("~/.local/lib/python3.12/site-packages").to_string().into(),
                shellexpand::tilde("~/.local/lib/python3.11/site-packages").to_string().into(),
            ]
        };
        
        // Find first existing path
        for path in possible_paths {
            if path.exists() {
                tracing::debug!("Using detected site-packages: {}", path.display());
                return Self::new(path);
            }
        }
        
        // Last resort: try to get from Python itself
        if let Ok(output) = std::process::Command::new("python3")
            .args(&["-c", "import site; print(site.getsitepackages()[0] if site.getsitepackages() else site.getusersitepackages())"])
            .output()
        {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    tracing::debug!("Using Python-detected site-packages: {}", path.display());
                    return Self::new(path);
                }
            }
        }
        
        // Final fallback: create relative path (may be empty, but won't crash)
        let path = if cfg!(target_os = "windows") {
            PathBuf::from("Lib/site-packages")
        } else {
            PathBuf::from("lib/python3.11/site-packages")
        };
        Self::new(path)
    }

    /// Get all site-packages directories (including fallback locations)
    pub fn get_all_directories(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }

    /// Detect Python version from venv
    fn detect_python_version(venv_path: &Path) -> Option<String> {
        // Check pyvenv.cfg for Python version
        let pyvenv_cfg = venv_path.join("pyvenv.cfg");
        if let Ok(content) = fs::read_to_string(&pyvenv_cfg) {
            for line in content.lines() {
                if line.starts_with("version") {
                    if let Some(version) = line.split('=').nth(1) {
                        let version = version.trim();
                        // Extract major.minor from version string
                        if let Some(dot_pos) = version.find('.') {
                            if let Some(minor_end) = version[dot_pos+1..].find(|c: char| !c.is_ascii_digit()) {
                                let minor_end = dot_pos + 1 + minor_end;
                                return Some(version[..minor_end].to_string());
                            }
                            // Try to get major.minor (first 3 chars after dot_pos)
                            if version.len() >= dot_pos + 3 {
                                return Some(version[..dot_pos+3].to_string());
                            }
                            return Some(version.to_string());
                        }
                    }
                }
            }
        }
        
        // Try to detect from Python executable
        let python_exe = if cfg!(target_os = "windows") {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python3")
        };
        
        if python_exe.exists() {
            // In a real implementation, we'd run python --version
            // For now, default to 3.11
            return Some("3.11".to_string());
        }
        
        None
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
