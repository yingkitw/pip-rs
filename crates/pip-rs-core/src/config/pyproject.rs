/// pyproject.toml parsing and management
use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PyProject {
    path: std::path::PathBuf,
    content: String,
}

impl PyProject {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("pyproject.toml not found at {:?}", path));
        }

        let content = fs::read_to_string(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            content,
        })
    }

    /// Get project name
    pub fn get_name(&self) -> Option<String> {
        self.get_value("project", "name")
    }

    /// Get project version
    pub fn get_version(&self) -> Option<String> {
        self.get_value("project", "version")
    }

    /// Get project description
    pub fn get_description(&self) -> Option<String> {
        self.get_value("project", "description")
    }

    /// Get project dependencies
    pub fn get_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();

        // Simple parsing for dependencies
        if let Some(start) = self.content.find("dependencies = [") {
            let rest = &self.content[start + 16..];
            if let Some(end) = rest.find(']') {
                let deps_str = &rest[..end];
                for line in deps_str.lines() {
                    let line = line.trim();
                    if line.starts_with('"') || line.starts_with('\'') {
                        let dep = line.trim_matches(|c| c == '"' || c == '\'' || c == ',');
                        if !dep.is_empty() {
                            deps.push(dep.to_string());
                        }
                    }
                }
            }
        }

        deps
    }

    /// Get optional dependencies
    pub fn get_optional_dependencies(&self) -> std::collections::HashMap<String, Vec<String>> {
        let mut optional = std::collections::HashMap::new();

        if let Some(start) = self.content.find("optional-dependencies") {
            let rest = &self.content[start..];
            // Simple parsing for optional dependencies
            // This is a simplified version - full TOML parsing would be better
            for line in rest.lines().take(20) {
                if line.contains('=') && line.contains('[') {
                    if let Some((key, _)) = line.split_once('=') {
                        let key = key.trim().to_string();
                        optional.insert(key, Vec::new());
                    }
                }
            }
        }

        optional
    }

    /// Get build system
    pub fn get_build_system(&self) -> Option<String> {
        self.get_value("build-system", "requires")
    }

    /// Helper to extract values
    fn get_value(&self, _section: &str, key: &str) -> Option<String> {
        let pattern = format!("{} = \"", key);
        
        if let Some(start) = self.content.find(&pattern) {
            let rest = &self.content[start + pattern.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pyproject_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyproject_path = temp_dir.path().join("pyproject.toml");

        let content = r#"
[project]
name = "test-package"
version = "0.1.0"
description = "A test package"
dependencies = [
    "requests>=2.28.0",
    "numpy>=1.20.0",
]
"#;

        fs::write(&pyproject_path, content)?;
        let pyproject = PyProject::load(&pyproject_path)?;

        assert_eq!(pyproject.get_name(), Some("test-package".to_string()));
        assert_eq!(pyproject.get_version(), Some("0.1.0".to_string()));
        assert_eq!(pyproject.get_description(), Some("A test package".to_string()));

        Ok(())
    }

    #[test]
    fn test_pyproject_dependencies() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyproject_path = temp_dir.path().join("pyproject.toml");

        let content = r#"
[project]
name = "test-package"
dependencies = [
    "requests>=2.28.0",
    "numpy>=1.20.0",
]
"#;

        fs::write(&pyproject_path, content)?;
        let pyproject = PyProject::load(&pyproject_path)?;
        let deps = pyproject.get_dependencies();

        assert!(deps.len() > 0);

        Ok(())
    }
}
