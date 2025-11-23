/// Egg-link file handling for editable installs
/// 
/// This module handles .egg-link files which are used for editable installs
/// and extracts project location information from them.

use std::path::{Path, PathBuf};

/// Egg-link information
#[derive(Clone, Debug)]
pub struct EggLinkInfo {
    pub project_location: PathBuf,
    pub sys_path_entries: Vec<PathBuf>,
    pub package_name: String,
}

impl EggLinkInfo {
    /// Parse egg-link file
    pub fn parse(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read egg-link file: {}", e))?;

        Self::parse_content(&content, path)
    }

    /// Parse egg-link content
    pub fn parse_content(content: &str, egg_link_path: &Path) -> Result<Self, String> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err("Empty egg-link file".to_string());
        }

        // First line is the project location (relative or absolute)
        let project_location_str = lines[0].trim();
        let project_location = if project_location_str.starts_with('/') {
            PathBuf::from(project_location_str)
        } else if project_location_str.starts_with('.') {
            // Relative to egg-link file location
            if let Some(parent) = egg_link_path.parent() {
                parent.join(project_location_str)
            } else {
                PathBuf::from(project_location_str)
            }
        } else {
            PathBuf::from(project_location_str)
        };

        // Remaining lines are sys.path entries
        let mut sys_path_entries = vec![];
        for line in &lines[1..] {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                sys_path_entries.push(PathBuf::from(trimmed));
            }
        }

        // Extract package name from egg-link filename
        let package_name = egg_link_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(EggLinkInfo {
            project_location,
            sys_path_entries,
            package_name,
        })
    }

    /// Get absolute project location
    pub fn get_absolute_location(&self) -> PathBuf {
        if self.project_location.is_absolute() {
            self.project_location.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(&self.project_location)
        }
    }

    /// Check if project location exists
    pub fn project_exists(&self) -> bool {
        self.get_absolute_location().exists()
    }

    /// Get setup.py location if it exists
    pub fn get_setup_py(&self) -> Option<PathBuf> {
        let setup_py = self.get_absolute_location().join("setup.py");
        if setup_py.exists() {
            Some(setup_py)
        } else {
            None
        }
    }

    /// Get pyproject.toml location if it exists
    pub fn get_pyproject_toml(&self) -> Option<PathBuf> {
        let pyproject = self.get_absolute_location().join("pyproject.toml");
        if pyproject.exists() {
            Some(pyproject)
        } else {
            None
        }
    }
}

/// Egg-link handler
pub struct EggLinkHandler;

impl EggLinkHandler {
    /// Find egg-link file for a package
    pub fn find_egg_link(package_name: &str, site_packages: &Path) -> Option<PathBuf> {
        let egg_link_path = site_packages.join(format!("{}.egg-link", package_name));
        if egg_link_path.exists() {
            return Some(egg_link_path);
        }

        // Try with normalized name
        let normalized = package_name.replace('-', "_").replace('.', "_");
        let egg_link_path = site_packages.join(format!("{}.egg-link", normalized));
        if egg_link_path.exists() {
            return Some(egg_link_path);
        }

        None
    }

    /// Extract project location from egg-link
    pub fn extract_location(egg_link_path: &Path) -> Result<PathBuf, String> {
        let info = EggLinkInfo::parse(egg_link_path)?;
        Ok(info.get_absolute_location())
    }

    /// Validate egg-link file
    pub fn validate(egg_link_path: &Path) -> Result<(), String> {
        let info = EggLinkInfo::parse(egg_link_path)?;

        if !info.project_exists() {
            return Err(format!(
                "Project location does not exist: {}",
                info.project_location.display()
            ));
        }

        Ok(())
    }

    /// Get all egg-link files in site-packages
    pub fn find_all_egg_links(site_packages: &Path) -> Result<Vec<EggLinkInfo>, String> {
        let mut egg_links = vec![];

        if !site_packages.exists() {
            return Ok(egg_links);
        }

        for entry in std::fs::read_dir(site_packages)
            .map_err(|e| format!("Failed to read site-packages: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.ends_with(".egg-link") {
                        if let Ok(info) = EggLinkInfo::parse(&path) {
                            egg_links.push(info);
                        }
                    }
                }
            }
        }

        Ok(egg_links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egg_link_info_parse_content() {
        let content = "/path/to/project\n/path/to/sys/entry\n";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        assert_eq!(info.project_location, PathBuf::from("/path/to/project"));
        assert_eq!(info.package_name, "package");
        assert_eq!(info.sys_path_entries.len(), 1);
    }

    #[test]
    fn test_egg_link_info_parse_relative_path() {
        let content = "../project\n";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        assert!(info.project_location.to_string_lossy().contains("project"));
    }

    #[test]
    fn test_egg_link_info_get_absolute_location() {
        let content = "/path/to/project\n";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        let abs_path = info.get_absolute_location();
        assert!(abs_path.is_absolute());
    }

    #[test]
    fn test_egg_link_info_with_comments() {
        let content = "/path/to/project\n# This is a comment\n/path/to/sys/entry\n";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        assert_eq!(info.sys_path_entries.len(), 1);
    }

    #[test]
    fn test_egg_link_handler_find_egg_link() {
        // This test would need a real site-packages directory
        // For now, just test that the function exists
        let site_packages = Path::new("/nonexistent");
        let result = EggLinkHandler::find_egg_link("package", site_packages);
        assert!(result.is_none());
    }

    #[test]
    fn test_egg_link_handler_extract_location() {
        let content = "/path/to/project\n";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        let location = info.get_absolute_location();
        assert!(location.is_absolute());
    }

    #[test]
    fn test_egg_link_info_empty_file() {
        let content = "";
        let egg_link_path = Path::new("/site-packages/package.egg-link");
        let result = EggLinkInfo::parse_content(content, egg_link_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_egg_link_info_package_name_extraction() {
        let content = "/path/to/project\n";
        let egg_link_path = Path::new("/site-packages/my-package.egg-link");
        let info = EggLinkInfo::parse_content(content, egg_link_path).unwrap();

        assert_eq!(info.package_name, "my-package");
    }
}
