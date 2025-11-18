/// Argument parser utilities
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub packages: Vec<String>,
    pub requirements: Option<PathBuf>,
    pub target: Option<PathBuf>,
    pub upgrade: bool,
    pub force_reinstall: bool,
    pub no_deps: bool,
}

#[derive(Debug, Clone)]
pub struct UninstallOptions {
    pub packages: Vec<String>,
    pub yes: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            requirements: None,
            target: None,
            upgrade: false,
            force_reinstall: false,
            no_deps: false,
        }
    }
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            yes: false,
        }
    }
}
