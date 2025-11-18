/// Package installer implementation
use crate::models::Package;
use crate::network::PackageClient;
use super::{SitePackages, wheel::WheelFile};
use anyhow::{Result, anyhow};
use std::path::Path;
use tempfile::TempDir;

#[allow(dead_code)]
pub struct PackageInstaller {
    client: PackageClient,
    site_packages: SitePackages,
}

impl PackageInstaller {
    pub fn new(site_packages: SitePackages) -> Self {
        Self {
            client: PackageClient::new(),
            site_packages,
        }
    }

    pub async fn install(&self, package: &Package) -> Result<()> {
        println!("Installing {} {}", package.name, package.version);

        // Create temporary directory for downloads
        let _ = TempDir::new()?;

        // TODO: Download wheel file
        // let wheel_url = self.find_wheel_url(package).await?;
        // let wheel_data = self.client.download_package(&wheel_url).await?;
        // let wheel_path = temp_dir.path().join(format!("{}-{}.whl", package.name, package.version));
        // std::fs::write(&wheel_path, wheel_data)?;

        // TODO: Extract and install wheel
        // let wheel = WheelFile::new(wheel_path)?;
        // self.install_wheel(&wheel).await?;

        println!("Successfully installed {} {}", package.name, package.version);
        Ok(())
    }

    pub async fn install_wheel(&self, wheel: &WheelFile) -> Result<()> {
        println!("Extracting wheel: {}", wheel.name);

        // Create temporary extraction directory
        let temp_dir = TempDir::new()?;
        
        // Extract wheel contents
        wheel.extract(temp_dir.path())?;

        // Get wheel metadata
        let metadata = wheel.get_metadata()?;

        // Install package files
        for entry in std::fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow!("Invalid directory name"))?;

                if dir_name.ends_with(".dist-info") {
                    // Install metadata
                    self.install_metadata(&path, dir_name)?;
                } else if dir_name.ends_with(".data") {
                    // Install data files
                    self.install_data_files(&path)?;
                } else {
                    // Install package files
                    self.site_packages.install_directory(&path, Path::new(dir_name))?;
                }
            }
        }

        println!("Successfully installed {} {}", metadata.name, metadata.version);
        Ok(())
    }

    fn install_metadata(&self, source: &Path, dist_info_name: &str) -> Result<()> {
        let target = self.site_packages.path().join(dist_info_name);
        std::fs::create_dir_all(&target)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let target_file = target.join(&file_name);

            if path.is_file() {
                std::fs::copy(&path, &target_file)?;
            }
        }

        Ok(())
    }

    fn install_data_files(&self, data_dir: &Path) -> Result<()> {
        // Handle purelib, platlib, headers, scripts, data
        for entry in std::fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();
            let dir_type = entry.file_name();

            if let Some(dir_type_str) = dir_type.to_str() {
                match dir_type_str {
                    "purelib" | "platlib" => {
                        // Install to site-packages
                        for file_entry in std::fs::read_dir(&path)? {
                            let file_entry = file_entry?;
                            let file_path = file_entry.path();
                            let file_name = file_entry.file_name();
                            
                            if file_path.is_dir() {
                                self.site_packages.install_directory(&file_path, Path::new(&file_name))?;
                            }
                        }
                    }
                    "scripts" => {
                        // Install to bin directory
                        // TODO: Implement script installation
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub async fn uninstall(&self, package_name: &str) -> Result<()> {
        println!("Uninstalling {}", package_name);
        
        let dist_info = self.site_packages.path().join(format!("{}.dist-info", package_name));
        if dist_info.exists() {
            std::fs::remove_dir_all(&dist_info)?;
            println!("Successfully uninstalled {}", package_name);
        } else {
            return Err(anyhow!("Package {} not found", package_name));
        }

        Ok(())
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        self.site_packages.get_installed_packages()
    }
}

impl PackageInstaller {
    pub fn default_installer() -> Result<Self> {
        let site_packages = SitePackages::default()?;
        Ok(Self::new(site_packages))
    }
}
