/// Direct URL support for package dependencies
/// 
/// This module handles direct URLs in package specifications,
/// enabling support for git+, file://, and other direct URL schemes.

use std::collections::HashMap;

/// Direct URL information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DirectUrl {
    pub url: String,
    pub url_type: DirectUrlType,
    pub subdirectory: Option<String>,
    pub editable: bool,
}

/// Type of direct URL
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DirectUrlType {
    /// Git repository
    Git,
    /// Mercurial repository
    Hg,
    /// Subversion repository
    Svn,
    /// Bazaar repository
    Bzr,
    /// Local file path
    File,
    /// HTTP/HTTPS URL
    Http,
    /// Other URL type
    Other(String),
}

impl DirectUrlType {
    /// Parse URL type from scheme
    pub fn from_scheme(scheme: &str) -> Self {
        match scheme {
            "git" | "git+http" | "git+https" | "git+ssh" | "git+git" | "git+file" => {
                DirectUrlType::Git
            }
            "hg" | "hg+http" | "hg+https" | "hg+static-http" | "hg+ssh" => {
                DirectUrlType::Hg
            }
            "svn" | "svn+http" | "svn+https" | "svn+ssh" | "svn+svn" => {
                DirectUrlType::Svn
            }
            "bzr" | "bzr+http" | "bzr+https" | "bzr+ssh" | "bzr+sftp" | "bzr+ftp" | "bzr+lp" => {
                DirectUrlType::Bzr
            }
            "file" => DirectUrlType::File,
            "http" | "https" => DirectUrlType::Http,
            other => DirectUrlType::Other(other.to_string()),
        }
    }

    /// Get scheme string
    pub fn scheme(&self) -> &str {
        match self {
            DirectUrlType::Git => "git+https",
            DirectUrlType::Hg => "hg+https",
            DirectUrlType::Svn => "svn+https",
            DirectUrlType::Bzr => "bzr+https",
            DirectUrlType::File => "file",
            DirectUrlType::Http => "https",
            DirectUrlType::Other(s) => s,
        }
    }
}

impl DirectUrl {
    /// Parse a direct URL string
    pub fn parse(url_str: &str) -> Option<Self> {
        // Handle editable flag
        let (editable, url_str) = if url_str.starts_with("-e ") {
            (true, &url_str[3..])
        } else {
            (false, url_str)
        };

        // Parse subdirectory
        let (url_str, subdirectory) = if let Some(pos) = url_str.find("#subdirectory=") {
            let (url, subdir) = url_str.split_at(pos);
            (url, Some(subdir[14..].to_string()))
        } else {
            (url_str, None)
        };

        // Extract scheme
        if let Some(pos) = url_str.find("://") {
            let scheme = &url_str[..pos];
            let url_type = DirectUrlType::from_scheme(scheme);

            Some(DirectUrl {
                url: url_str.to_string(),
                url_type,
                subdirectory,
                editable,
            })
        } else {
            None
        }
    }

    /// Check if this direct URL conflicts with another
    pub fn conflicts_with(&self, other: &DirectUrl) -> bool {
        // Different URLs are considered conflicting
        self.url != other.url
    }

    /// Get a normalized representation for comparison
    pub fn normalize(&self) -> String {
        format!("{}#{}", self.url, self.editable)
    }

    /// Check if URL is editable install
    pub fn is_editable(&self) -> bool {
        self.editable
    }

    /// Check if URL is a VCS URL
    pub fn is_vcs(&self) -> bool {
        matches!(
            self.url_type,
            DirectUrlType::Git | DirectUrlType::Hg | DirectUrlType::Svn | DirectUrlType::Bzr
        )
    }

    /// Check if URL is a local file
    pub fn is_local(&self) -> bool {
        matches!(self.url_type, DirectUrlType::File)
    }
}

/// Direct URL conflict detector
pub struct DirectUrlConflictDetector {
    urls: HashMap<String, DirectUrl>,
}

impl DirectUrlConflictDetector {
    pub fn new() -> Self {
        Self {
            urls: HashMap::new(),
        }
    }

    /// Register a direct URL
    pub fn register(&mut self, package_name: &str, direct_url: DirectUrl) {
        self.urls
            .insert(package_name.to_lowercase(), direct_url);
    }

    /// Check for conflicts
    pub fn check_conflict(&self, package_name: &str, direct_url: &DirectUrl) -> Option<String> {
        let key = package_name.to_lowercase();
        if let Some(existing) = self.urls.get(&key) {
            if existing.conflicts_with(direct_url) {
                return Some(format!(
                    "Conflicting direct URLs for {}: {} vs {}",
                    package_name, existing.url, direct_url.url
                ));
            }
        }
        None
    }

    /// Get all registered URLs
    pub fn get_urls(&self) -> &HashMap<String, DirectUrl> {
        &self.urls
    }

    /// Clear all registered URLs
    pub fn clear(&mut self) {
        self.urls.clear();
    }
}

impl Default for DirectUrlConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_url_parse_git() {
        let url = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        assert_eq!(url.url_type, DirectUrlType::Git);
        assert!(!url.editable);
    }

    #[test]
    fn test_direct_url_parse_file() {
        let url = DirectUrl::parse("file:///path/to/package").unwrap();
        assert_eq!(url.url_type, DirectUrlType::File);
    }

    #[test]
    fn test_direct_url_parse_editable() {
        let url = DirectUrl::parse("-e git+https://github.com/user/repo.git").unwrap();
        assert!(url.editable);
    }

    #[test]
    fn test_direct_url_parse_subdirectory() {
        let url = DirectUrl::parse("git+https://github.com/user/repo.git#subdirectory=subdir")
            .unwrap();
        assert_eq!(url.subdirectory, Some("subdir".to_string()));
    }

    #[test]
    fn test_direct_url_conflicts() {
        let url1 = DirectUrl::parse("git+https://github.com/user/repo1.git").unwrap();
        let url2 = DirectUrl::parse("git+https://github.com/user/repo2.git").unwrap();
        assert!(url1.conflicts_with(&url2));
    }

    #[test]
    fn test_direct_url_no_conflict() {
        let url1 = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        let url2 = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        assert!(!url1.conflicts_with(&url2));
    }

    #[test]
    fn test_direct_url_is_vcs() {
        let git_url = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        assert!(git_url.is_vcs());

        let file_url = DirectUrl::parse("file:///path/to/package").unwrap();
        assert!(!file_url.is_vcs());
    }

    #[test]
    fn test_direct_url_is_local() {
        let file_url = DirectUrl::parse("file:///path/to/package").unwrap();
        assert!(file_url.is_local());

        let git_url = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        assert!(!git_url.is_local());
    }

    #[test]
    fn test_conflict_detector_register() {
        let mut detector = DirectUrlConflictDetector::new();
        let url = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        detector.register("mypackage", url);
        assert_eq!(detector.get_urls().len(), 1);
    }

    #[test]
    fn test_conflict_detector_check_conflict() {
        let mut detector = DirectUrlConflictDetector::new();
        let url1 = DirectUrl::parse("git+https://github.com/user/repo1.git").unwrap();
        let url2 = DirectUrl::parse("git+https://github.com/user/repo2.git").unwrap();

        detector.register("mypackage", url1);
        let conflict = detector.check_conflict("mypackage", &url2);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_conflict_detector_no_conflict() {
        let mut detector = DirectUrlConflictDetector::new();
        let url1 = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();
        let url2 = DirectUrl::parse("git+https://github.com/user/repo.git").unwrap();

        detector.register("mypackage", url1);
        let conflict = detector.check_conflict("mypackage", &url2);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_url_type_from_scheme() {
        assert_eq!(DirectUrlType::from_scheme("git"), DirectUrlType::Git);
        assert_eq!(DirectUrlType::from_scheme("git+https"), DirectUrlType::Git);
        assert_eq!(DirectUrlType::from_scheme("file"), DirectUrlType::File);
        assert_eq!(DirectUrlType::from_scheme("https"), DirectUrlType::Http);
    }
}
