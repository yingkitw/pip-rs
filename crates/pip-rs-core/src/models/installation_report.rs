/// Installation report with environment override support
/// 
/// This module handles installation reports with support for environment
/// overrides (--python-version, --platform, etc.)

use std::collections::HashMap;

/// Installation report
#[derive(Clone, Debug)]
pub struct InstallationReport {
    pub installed_packages: Vec<InstalledPackage>,
    pub environment: EnvironmentInfo,
    pub environment_override: Option<EnvironmentOverride>,
    pub timestamp: String,
}

/// Installed package information
#[derive(Clone, Debug)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub location: String,
    pub editable: bool,
    pub direct_url: Option<String>,
}

/// Environment information
#[derive(Clone, Debug)]
pub struct EnvironmentInfo {
    pub python_version: String,
    pub platform: String,
    pub implementation: String,
    pub architecture: String,
    pub os_name: String,
    pub sys_platform: String,
}

impl EnvironmentInfo {
    /// Create default environment info
    pub fn default() -> Self {
        Self {
            python_version: format!("{}.{}", 3, 11),
            platform: Self::get_platform(),
            implementation: "cpython".to_string(),
            architecture: Self::get_architecture(),
            os_name: Self::get_os_name(),
            sys_platform: Self::get_platform(),
        }
    }

    fn get_platform() -> String {
        if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "win32".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn get_architecture() -> String {
        if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn get_os_name() -> String {
        if cfg!(unix) {
            "posix".to_string()
        } else if cfg!(windows) {
            "nt".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Convert to map for serialization
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("python_version".to_string(), self.python_version.clone());
        map.insert("platform".to_string(), self.platform.clone());
        map.insert("implementation_name".to_string(), self.implementation.clone());
        map.insert("platform_machine".to_string(), self.architecture.clone());
        map.insert("os_name".to_string(), self.os_name.clone());
        map.insert("sys_platform".to_string(), self.sys_platform.clone());
        map
    }
}

/// Environment override for cross-platform builds
#[derive(Clone, Debug)]
pub struct EnvironmentOverride {
    pub python_version: Option<String>,
    pub platform: Option<String>,
    pub implementation: Option<String>,
    pub architecture: Option<String>,
}

impl EnvironmentOverride {
    pub fn new() -> Self {
        Self {
            python_version: None,
            platform: None,
            implementation: None,
            architecture: None,
        }
    }

    /// Apply override to environment info
    pub fn apply(&self, env: &mut EnvironmentInfo) {
        if let Some(version) = &self.python_version {
            env.python_version = version.clone();
        }
        if let Some(platform) = &self.platform {
            env.platform = platform.clone();
            env.sys_platform = platform.clone();
        }
        if let Some(impl_name) = &self.implementation {
            env.implementation = impl_name.clone();
        }
        if let Some(arch) = &self.architecture {
            env.architecture = arch.clone();
        }
    }

    /// Check if any override is set
    pub fn is_set(&self) -> bool {
        self.python_version.is_some()
            || self.platform.is_some()
            || self.implementation.is_some()
            || self.architecture.is_some()
    }

    /// Get override description
    pub fn describe(&self) -> String {
        let mut parts = vec![];

        if let Some(version) = &self.python_version {
            parts.push(format!("python_version={}", version));
        }
        if let Some(platform) = &self.platform {
            parts.push(format!("platform={}", platform));
        }
        if let Some(impl_name) = &self.implementation {
            parts.push(format!("implementation={}", impl_name));
        }
        if let Some(arch) = &self.architecture {
            parts.push(format!("architecture={}", arch));
        }

        if parts.is_empty() {
            "no overrides".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl Default for EnvironmentOverride {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallationReport {
    pub fn new() -> Self {
        Self {
            installed_packages: vec![],
            environment: EnvironmentInfo::default(),
            environment_override: None,
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }

    /// Add installed package
    pub fn add_package(&mut self, package: InstalledPackage) {
        self.installed_packages.push(package);
    }

    /// Set environment override
    pub fn set_override(&mut self, override_: EnvironmentOverride) {
        if override_.is_set() {
            override_.apply(&mut self.environment);
            self.environment_override = Some(override_);
        }
    }

    /// Get effective environment (with overrides applied)
    pub fn effective_environment(&self) -> EnvironmentInfo {
        self.environment.clone()
    }

    /// Convert to JSON-serializable map
    pub fn to_map(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();

        // Installed packages
        let packages: Vec<_> = self
            .installed_packages
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "version": p.version,
                    "location": p.location,
                    "editable": p.editable,
                    "direct_url": p.direct_url,
                })
            })
            .collect();
        map.insert(
            "installed".to_string(),
            serde_json::Value::Array(packages),
        );

        // Environment
        let env_map = self.environment.to_map();
        let env_json: HashMap<String, serde_json::Value> = env_map
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        map.insert("environment".to_string(), serde_json::to_value(env_json).unwrap());

        // Environment override
        if let Some(override_) = &self.environment_override {
            map.insert(
                "environment_override".to_string(),
                serde_json::Value::String(override_.describe()),
            );
        }

        // Timestamp
        map.insert(
            "timestamp".to_string(),
            serde_json::Value::String(self.timestamp.clone()),
        );

        map
    }
}

impl Default for InstallationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_info_default() {
        let env = EnvironmentInfo::default();
        assert!(!env.python_version.is_empty());
        assert!(!env.platform.is_empty());
        assert_eq!(env.implementation, "cpython");
    }

    #[test]
    fn test_environment_info_to_map() {
        let env = EnvironmentInfo::default();
        let map = env.to_map();
        assert!(map.contains_key("python_version"));
        assert!(map.contains_key("platform"));
        assert!(map.contains_key("implementation_name"));
    }

    #[test]
    fn test_environment_override_new() {
        let override_ = EnvironmentOverride::new();
        assert!(!override_.is_set());
    }

    #[test]
    fn test_environment_override_apply() {
        let mut override_ = EnvironmentOverride::new();
        override_.python_version = Some("3.9".to_string());
        override_.platform = Some("linux".to_string());

        let mut env = EnvironmentInfo::default();
        override_.apply(&mut env);

        assert_eq!(env.python_version, "3.9");
        assert_eq!(env.platform, "linux");
    }

    #[test]
    fn test_environment_override_is_set() {
        let mut override_ = EnvironmentOverride::new();
        assert!(!override_.is_set());

        override_.python_version = Some("3.9".to_string());
        assert!(override_.is_set());
    }

    #[test]
    fn test_environment_override_describe() {
        let mut override_ = EnvironmentOverride::new();
        override_.python_version = Some("3.9".to_string());
        override_.platform = Some("linux".to_string());

        let desc = override_.describe();
        assert!(desc.contains("python_version=3.9"));
        assert!(desc.contains("platform=linux"));
    }

    #[test]
    fn test_installation_report_new() {
        let report = InstallationReport::new();
        assert!(report.installed_packages.is_empty());
        assert!(!report.timestamp.is_empty());
    }

    #[test]
    fn test_installation_report_add_package() {
        let mut report = InstallationReport::new();
        let pkg = InstalledPackage {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            location: "/usr/lib/python3.11/site-packages".to_string(),
            editable: false,
            direct_url: None,
        };

        report.add_package(pkg);
        assert_eq!(report.installed_packages.len(), 1);
    }

    #[test]
    fn test_installation_report_set_override() {
        let mut report = InstallationReport::new();
        let mut override_ = EnvironmentOverride::new();
        override_.python_version = Some("3.9".to_string());

        report.set_override(override_);
        assert!(report.environment_override.is_some());
        assert_eq!(report.environment.python_version, "3.9");
    }

    #[test]
    fn test_installation_report_to_map() {
        let mut report = InstallationReport::new();
        let pkg = InstalledPackage {
            name: "requests".to_string(),
            version: "2.28.0".to_string(),
            location: "/usr/lib/python3.11/site-packages".to_string(),
            editable: false,
            direct_url: None,
        };

        report.add_package(pkg);
        let map = report.to_map();

        assert!(map.contains_key("installed"));
        assert!(map.contains_key("environment"));
        assert!(map.contains_key("timestamp"));
    }
}
