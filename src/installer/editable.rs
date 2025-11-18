/// Editable package installation support
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct EditableInstall {
    project_path: PathBuf,
    site_packages: PathBuf,
}

impl EditableInstall {
    pub fn new(project_path: PathBuf, site_packages: PathBuf) -> Self {
        Self {
            project_path,
            site_packages,
        }
    }

    /// Install a package in editable mode
    pub fn install(&self) -> Result<()> {
        // Verify project has pyproject.toml or setup.py
        if !self.has_build_config() {
            return Err(anyhow!("No pyproject.toml or setup.py found"));
        }

        // Create .pth file for editable install
        self.create_pth_file()?;

        // Create .dist-info directory
        self.create_dist_info()?;

        Ok(())
    }

    /// Check if project has build configuration
    fn has_build_config(&self) -> bool {
        self.project_path.join("pyproject.toml").exists()
            || self.project_path.join("setup.py").exists()
    }

    /// Create .pth file for editable install
    fn create_pth_file(&self) -> Result<()> {
        let project_name = self.project_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        let pth_content = format!("{}\n", self.project_path.display());
        let pth_file = self.site_packages.join(format!("__{}_path__.pth", project_name));

        fs::write(&pth_file, pth_content)?;
        Ok(())
    }

    /// Create .dist-info directory for editable install
    fn create_dist_info(&self) -> Result<()> {
        let project_name = self.project_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        let dist_info = self.site_packages.join(format!("{}.dist-info", project_name));
        fs::create_dir_all(&dist_info)?;

        // Create METADATA file
        let metadata = format!(
            "Metadata-Version: 2.1\n\
             Name: {}\n\
             Version: 0.0.0\n\
             Summary: Editable install\n\
             Home-page: {}\n",
            project_name,
            self.project_path.display()
        );
        fs::write(dist_info.join("METADATA"), metadata)?;

        // Create WHEEL file
        let wheel = "Wheel-Version: 1.0\n\
                     Generator: pip-rs\n\
                     Root-Is-Purelib: true\n\
                     Tag: py3-none-any\n";
        fs::write(dist_info.join("WHEEL"), wheel)?;

        // Create RECORD file
        let record = format!(
            "{}/METADATA,,\n\
             {}/WHEEL,,\n\
             {}/RECORD,,\n",
            dist_info.display(),
            dist_info.display(),
            dist_info.display()
        );
        fs::write(dist_info.join("RECORD"), record)?;

        Ok(())
    }

    /// Uninstall editable package
    pub fn uninstall(&self) -> Result<()> {
        let project_name = self.project_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        // Remove .pth file
        let pth_file = self.site_packages.join(format!("__{}_path__.pth", project_name));
        if pth_file.exists() {
            fs::remove_file(&pth_file)?;
        }

        // Remove .dist-info directory
        let dist_info = self.site_packages.join(format!("{}.dist-info", project_name));
        if dist_info.exists() {
            fs::remove_dir_all(&dist_info)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_editable_install() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_dir = temp_dir.path().join("project");
        let site_packages = temp_dir.path().join("site-packages");

        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(&site_packages)?;

        // Create pyproject.toml
        fs::write(project_dir.join("pyproject.toml"), "[project]\nname = \"test\"\n")?;

        let editable = EditableInstall::new(project_dir, site_packages.clone());
        editable.install()?;

        // Verify .pth file was created
        let pth_files: Vec<_> = fs::read_dir(&site_packages)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "pth"))
            .collect();

        assert!(!pth_files.is_empty());
        Ok(())
    }
}
