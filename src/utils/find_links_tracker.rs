/// Find-links tracking with relative path support
/// 
/// This module tracks find-links sources and supports relative paths
/// relative to the requirements file location.

use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Find-links source
#[derive(Clone, Debug)]
pub struct FindLinksSource {
    pub url: String,
    pub source_type: FindLinksType,
    pub is_relative: bool,
    pub base_path: Option<PathBuf>,
}

/// Type of find-links source
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FindLinksType {
    /// Local file path
    Local,
    /// HTTP/HTTPS URL
    Http,
    /// VCS URL (git, hg, svn, bzr)
    Vcs,
    /// Unknown type
    Unknown,
}

impl FindLinksSource {
    /// Parse a find-links source
    pub fn parse(url: &str, base_path: Option<&Path>) -> Self {
        let (is_relative, source_type) = Self::detect_type(url);

        let base_path = if is_relative {
            base_path.map(|p| p.to_path_buf())
        } else {
            None
        };

        Self {
            url: url.to_string(),
            source_type,
            is_relative,
            base_path,
        }
    }

    /// Detect the type of find-links source
    fn detect_type(url: &str) -> (bool, FindLinksType) {
        // Check for VCS URLs
        if url.starts_with("git+") || url.starts_with("hg+") || url.starts_with("svn+")
            || url.starts_with("bzr+")
        {
            return (false, FindLinksType::Vcs);
        }

        // Check for HTTP/HTTPS URLs
        if url.starts_with("http://") || url.starts_with("https://") {
            return (false, FindLinksType::Http);
        }

        // Check for local paths
        if url.starts_with("/") || url.starts_with("./") || url.starts_with("../")
            || url.contains("\\")
        {
            // Relative or absolute path
            let is_relative = !url.starts_with("/") && !url.starts_with("\\");
            return (is_relative, FindLinksType::Local);
        }

        // Default to local relative path
        (true, FindLinksType::Local)
    }

    /// Get absolute path for relative find-links
    pub fn get_absolute_path(&self) -> Option<PathBuf> {
        if self.is_relative {
            if let Some(base) = &self.base_path {
                return Some(base.join(&self.url));
            }
        } else if self.source_type == FindLinksType::Local {
            return Some(PathBuf::from(&self.url));
        }

        None
    }

    /// Check if the find-links source exists
    pub fn exists(&self) -> bool {
        if let Some(path) = self.get_absolute_path() {
            path.exists()
        } else {
            false
        }
    }

    /// Get description of the source
    pub fn describe(&self) -> String {
        match self.source_type {
            FindLinksType::Local => {
                if self.is_relative {
                    format!("local (relative): {}", self.url)
                } else {
                    format!("local (absolute): {}", self.url)
                }
            }
            FindLinksType::Http => format!("http: {}", self.url),
            FindLinksType::Vcs => format!("vcs: {}", self.url),
            FindLinksType::Unknown => format!("unknown: {}", self.url),
        }
    }
}

/// Find-links tracker
pub struct FindLinksTracker {
    sources: Vec<FindLinksSource>,
    base_path: Option<PathBuf>,
}

impl FindLinksTracker {
    pub fn new(base_path: Option<PathBuf>) -> Self {
        Self {
            sources: vec![],
            base_path,
        }
    }

    /// Add a find-links source
    pub fn add_source(&mut self, url: &str) {
        let source = FindLinksSource::parse(url, self.base_path.as_deref());
        self.sources.push(source);
    }

    /// Get all sources
    pub fn get_sources(&self) -> &[FindLinksSource] {
        &self.sources
    }

    /// Get local sources
    pub fn get_local_sources(&self) -> Vec<&FindLinksSource> {
        self.sources
            .iter()
            .filter(|s| s.source_type == FindLinksType::Local)
            .collect()
    }

    /// Get HTTP sources
    pub fn get_http_sources(&self) -> Vec<&FindLinksSource> {
        self.sources
            .iter()
            .filter(|s| s.source_type == FindLinksType::Http)
            .collect()
    }

    /// Get VCS sources
    pub fn get_vcs_sources(&self) -> Vec<&FindLinksSource> {
        self.sources
            .iter()
            .filter(|s| s.source_type == FindLinksType::Vcs)
            .collect()
    }

    /// Validate all sources
    pub fn validate(&self) -> Result<(), String> {
        for (i, source) in self.sources.iter().enumerate() {
            if source.source_type == FindLinksType::Local && !source.exists() {
                return Err(format!(
                    "Find-links source {} does not exist: {}",
                    i, source.url
                ));
            }
        }
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> FindLinksStats {
        let total = self.sources.len();
        let local = self.get_local_sources().len();
        let http = self.get_http_sources().len();
        let vcs = self.get_vcs_sources().len();

        FindLinksStats {
            total,
            local,
            http,
            vcs,
        }
    }
}

/// Find-links statistics
#[derive(Debug, Clone)]
pub struct FindLinksStats {
    pub total: usize,
    pub local: usize,
    pub http: usize,
    pub vcs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_links_source_parse_http() {
        let source = FindLinksSource::parse("https://example.com/packages", None);
        assert_eq!(source.source_type, FindLinksType::Http);
        assert!(!source.is_relative);
    }

    #[test]
    fn test_find_links_source_parse_local_relative() {
        let source = FindLinksSource::parse("./packages", None);
        assert_eq!(source.source_type, FindLinksType::Local);
        assert!(source.is_relative);
    }

    #[test]
    fn test_find_links_source_parse_local_absolute() {
        let source = FindLinksSource::parse("/usr/local/packages", None);
        assert_eq!(source.source_type, FindLinksType::Local);
        assert!(!source.is_relative);
    }

    #[test]
    fn test_find_links_source_parse_vcs() {
        let source = FindLinksSource::parse("git+https://github.com/user/repo.git", None);
        assert_eq!(source.source_type, FindLinksType::Vcs);
    }

    #[test]
    fn test_find_links_source_describe() {
        let source = FindLinksSource::parse("https://example.com/packages", None);
        let desc = source.describe();
        assert!(desc.contains("http"));
    }

    #[test]
    fn test_find_links_tracker_new() {
        let tracker = FindLinksTracker::new(None);
        assert_eq!(tracker.get_sources().len(), 0);
    }

    #[test]
    fn test_find_links_tracker_add_source() {
        let mut tracker = FindLinksTracker::new(None);
        tracker.add_source("https://example.com/packages");
        assert_eq!(tracker.get_sources().len(), 1);
    }

    #[test]
    fn test_find_links_tracker_get_http_sources() {
        let mut tracker = FindLinksTracker::new(None);
        tracker.add_source("https://example.com/packages");
        tracker.add_source("./local");
        assert_eq!(tracker.get_http_sources().len(), 1);
    }

    #[test]
    fn test_find_links_tracker_get_local_sources() {
        let mut tracker = FindLinksTracker::new(None);
        tracker.add_source("https://example.com/packages");
        tracker.add_source("./local");
        assert_eq!(tracker.get_local_sources().len(), 1);
    }

    #[test]
    fn test_find_links_tracker_stats() {
        let mut tracker = FindLinksTracker::new(None);
        tracker.add_source("https://example.com/packages");
        tracker.add_source("./local");
        tracker.add_source("git+https://github.com/user/repo.git");

        let stats = tracker.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.http, 1);
        assert_eq!(stats.local, 1);
        assert_eq!(stats.vcs, 1);
    }

    #[test]
    fn test_find_links_source_get_absolute_path() {
        let base = PathBuf::from("/home/user");
        let source = FindLinksSource::parse("packages", Some(&base));
        let abs_path = source.get_absolute_path();
        assert!(abs_path.is_some());
        assert_eq!(abs_path.unwrap(), PathBuf::from("/home/user/packages"));
    }
}
