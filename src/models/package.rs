/// Package representation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub home_page: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub requires_python: Option<String>,
    pub requires_dist: Vec<String>,
    pub classifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub releases: HashMap<String, Vec<Distribution>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub filename: String,
    pub url: String,
    pub hashes: HashMap<String, String>,
    pub requires_python: Option<String>,
    pub yanked: bool,
}

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            summary: None,
            home_page: None,
            author: None,
            license: None,
            requires_python: None,
            requires_dist: Vec::new(),
            classifiers: Vec::new(),
        }
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn with_requires(mut self, requires: Vec<String>) -> Self {
        self.requires_dist = requires;
        self
    }
}
