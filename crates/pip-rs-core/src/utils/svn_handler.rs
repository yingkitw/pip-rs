/// SVN (Subversion) entries error handling
/// 
/// This module handles SVN-related errors and provides utilities for
/// working with SVN repositories in editable installs.

use std::path::{Path, PathBuf};

/// SVN error types
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SvnError {
    /// SVN entries file not found
    EntriesNotFound,
    /// Invalid SVN entries format
    InvalidFormat(String),
    /// SVN command failed
    CommandFailed(String),
    /// SVN not installed
    NotInstalled,
    /// Permission denied
    PermissionDenied,
    /// Unknown error
    Unknown(String),
}

impl SvnError {
    /// Get error description
    pub fn description(&self) -> String {
        match self {
            SvnError::EntriesNotFound => "SVN entries file not found".to_string(),
            SvnError::InvalidFormat(msg) => format!("Invalid SVN entries format: {}", msg),
            SvnError::CommandFailed(msg) => format!("SVN command failed: {}", msg),
            SvnError::NotInstalled => "SVN is not installed or not in PATH".to_string(),
            SvnError::PermissionDenied => "Permission denied accessing SVN entries".to_string(),
            SvnError::Unknown(msg) => format!("Unknown SVN error: {}", msg),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SvnError::EntriesNotFound | SvnError::InvalidFormat(_) | SvnError::CommandFailed(_)
        )
    }
}

/// SVN entries information
#[derive(Clone, Debug)]
pub struct SvnEntries {
    pub repository_url: String,
    pub revision: String,
    pub directory: PathBuf,
}

impl SvnEntries {
    /// Parse SVN entries file
    pub fn parse(path: &Path) -> Result<Self, SvnError> {
        if !path.exists() {
            return Err(SvnError::EntriesNotFound);
        }

        let content = std::fs::read_to_string(path)
            .map_err(|_| SvnError::InvalidFormat("Failed to read entries file".to_string()))?;

        Self::parse_content(&content, path)
    }

    /// Parse SVN entries content
    pub fn parse_content(content: &str, entries_path: &Path) -> Result<Self, SvnError> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(SvnError::InvalidFormat("Empty entries file".to_string()));
        }

        // SVN entries format (XML-based in newer versions, text-based in older)
        // Try to extract repository URL and revision
        let mut repository_url = String::new();
        let mut revision = String::new();

        // Look for URL in entries
        for line in &lines {
            if line.contains("url=") {
                if let Some(start) = line.find("url=\"") {
                    if let Some(end) = line[start + 5..].find('"') {
                        repository_url = line[start + 5..start + 5 + end].to_string();
                    }
                }
            }
            if line.contains("revision=") {
                if let Some(start) = line.find("revision=\"") {
                    if let Some(end) = line[start + 10..].find('"') {
                        revision = line[start + 10..start + 10 + end].to_string();
                    }
                }
            }
        }

        // Fallback: use first non-empty line as URL if not found
        if repository_url.is_empty() {
            for line in &lines {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    repository_url = trimmed.to_string();
                    break;
                }
            }
        }

        if repository_url.is_empty() {
            return Err(SvnError::InvalidFormat(
                "Could not extract repository URL".to_string(),
            ));
        }

        let directory = entries_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        Ok(SvnEntries {
            repository_url,
            revision,
            directory,
        })
    }

    /// Get SVN info
    pub fn get_info(&self) -> String {
        format!(
            "SVN Repository: {}\nRevision: {}\nDirectory: {}",
            self.repository_url,
            if self.revision.is_empty() {
                "unknown".to_string()
            } else {
                self.revision.clone()
            },
            self.directory.display()
        )
    }
}

/// SVN handler
pub struct SvnHandler;

impl SvnHandler {
    /// Check if directory is SVN repository
    pub fn is_svn_repo(path: &Path) -> bool {
        let svn_dir = path.join(".svn");
        svn_dir.exists() && svn_dir.is_dir()
    }

    /// Find SVN entries file
    pub fn find_entries(path: &Path) -> Option<PathBuf> {
        let entries_path = path.join(".svn").join("entries");
        if entries_path.exists() {
            return Some(entries_path);
        }

        // Try alternative location for older SVN versions
        let entries_path = path.join(".svn").join("wc.db");
        if entries_path.exists() {
            return Some(entries_path);
        }

        None
    }

    /// Extract repository URL from SVN directory
    pub fn get_repository_url(path: &Path) -> Result<String, SvnError> {
        if !Self::is_svn_repo(path) {
            return Err(SvnError::EntriesNotFound);
        }

        if let Some(entries_path) = Self::find_entries(path) {
            let entries = SvnEntries::parse(&entries_path)?;
            Ok(entries.repository_url)
        } else {
            Err(SvnError::EntriesNotFound)
        }
    }

    /// Validate SVN entries
    pub fn validate_entries(path: &Path) -> Result<(), SvnError> {
        if let Some(entries_path) = Self::find_entries(path) {
            SvnEntries::parse(&entries_path)?;
            Ok(())
        } else {
            Err(SvnError::EntriesNotFound)
        }
    }

    /// Handle SVN error gracefully
    pub fn handle_error(error: &SvnError) -> String {
        if error.is_recoverable() {
            format!("Warning: {}", error.description())
        } else {
            format!("Error: {}", error.description())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svn_error_description() {
        let error = SvnError::EntriesNotFound;
        assert!(!error.description().is_empty());
    }

    #[test]
    fn test_svn_error_is_recoverable() {
        assert!(SvnError::EntriesNotFound.is_recoverable());
        assert!(SvnError::InvalidFormat("test".to_string()).is_recoverable());
        assert!(SvnError::CommandFailed("test".to_string()).is_recoverable());
        assert!(!SvnError::NotInstalled.is_recoverable());
    }

    #[test]
    fn test_svn_entries_parse_content() {
        let content = "12\ndir\n\nhttp://example.com/repo\n";
        let path = Path::new("/path/.svn/entries");
        let result = SvnEntries::parse_content(content, path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_svn_entries_parse_empty() {
        let content = "";
        let path = Path::new("/path/.svn/entries");
        let result = SvnEntries::parse_content(content, path);
        assert!(result.is_err());
    }

    #[test]
    fn test_svn_entries_get_info() {
        let entries = SvnEntries {
            repository_url: "http://example.com/repo".to_string(),
            revision: "123".to_string(),
            directory: PathBuf::from("/path"),
        };
        let info = entries.get_info();
        assert!(info.contains("http://example.com/repo"));
        assert!(info.contains("123"));
    }

    #[test]
    fn test_svn_handler_is_svn_repo() {
        // This test would need a real .svn directory
        // For now, just test that the function exists
        let path = Path::new("/nonexistent");
        assert!(!SvnHandler::is_svn_repo(path));
    }

    #[test]
    fn test_svn_handler_handle_error() {
        let error = SvnError::EntriesNotFound;
        let message = SvnHandler::handle_error(&error);
        assert!(message.contains("Warning"));
    }

    #[test]
    fn test_svn_error_equality() {
        let error1 = SvnError::EntriesNotFound;
        let error2 = SvnError::EntriesNotFound;
        assert_eq!(error1, error2);
    }
}
