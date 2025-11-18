/// Package metadata handling
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub home_page: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub classifiers: Vec<String>,
    pub requires_python: Option<String>,
    pub requires_dist: Vec<String>,
    pub provides_extra: Vec<String>,
}

impl Metadata {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            summary: None,
            home_page: None,
            author: None,
            author_email: None,
            license: None,
            description: None,
            keywords: None,
            classifiers: Vec::new(),
            requires_python: None,
            requires_dist: Vec::new(),
            provides_extra: Vec::new(),
        }
    }
}
