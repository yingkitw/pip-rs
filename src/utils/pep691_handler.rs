/// PEP 691 support for file:// URLs
/// 
/// This module implements PEP 691 (JSON-based simple API) support
/// for file:// URLs, enabling local package indexes.

use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// PEP 691 file URL information
#[derive(Clone, Debug)]
pub struct FileUrlInfo {
    pub path: PathBuf,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub requires_python: Option<String>,
    pub yanked: bool,
}

impl FileUrlInfo {
    /// Create new file URL info
    pub fn new(path: PathBuf, filename: String) -> Self {
        Self {
            path,
            filename,
            hashes: HashMap::new(),
            requires_python: None,
            yanked: false,
        }
    }

    /// Add hash
    pub fn add_hash(&mut self, algorithm: &str, value: &str) {
        self.hashes.insert(algorithm.to_lowercase(), value.to_string());
    }

    /// Set Python requirement
    pub fn set_requires_python(&mut self, version_spec: &str) {
        self.requires_python = Some(version_spec.to_string());
    }

    /// Set yanked status
    pub fn set_yanked(&mut self, yanked: bool) {
        self.yanked = yanked;
    }

    /// Get file URL
    pub fn get_file_url(&self) -> String {
        format!("file://{}", self.path.display())
    }

    /// Check if file exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Get file size
    pub fn get_size(&self) -> Option<u64> {
        std::fs::metadata(&self.path)
            .ok()
            .map(|m| m.len())
    }
}

/// PEP 691 file index
#[derive(Clone, Debug)]
pub struct FileIndex {
    pub base_path: PathBuf,
    pub files: Vec<FileUrlInfo>,
}

impl FileIndex {
    /// Create new file index
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            files: vec![],
        }
    }

    /// Add file to index
    pub fn add_file(&mut self, file_info: FileUrlInfo) {
        self.files.push(file_info);
    }

    /// Find files by package name
    pub fn find_by_package(&self, package_name: &str) -> Vec<&FileUrlInfo> {
        self.files
            .iter()
            .filter(|f| f.filename.contains(package_name))
            .collect()
    }

    /// Find files by pattern
    pub fn find_by_pattern(&self, pattern: &str) -> Vec<&FileUrlInfo> {
        self.files
            .iter()
            .filter(|f| f.filename.contains(pattern))
            .collect()
    }

    /// Get all wheel files
    pub fn get_wheels(&self) -> Vec<&FileUrlInfo> {
        self.files
            .iter()
            .filter(|f| f.filename.ends_with(".whl"))
            .collect()
    }

    /// Get all source distributions
    pub fn get_sdists(&self) -> Vec<&FileUrlInfo> {
        self.files
            .iter()
            .filter(|f| {
                f.filename.ends_with(".tar.gz")
                    || f.filename.ends_with(".zip")
                    || f.filename.ends_with(".tar.bz2")
            })
            .collect()
    }

    /// Get non-yanked files
    pub fn get_available(&self) -> Vec<&FileUrlInfo> {
        self.files.iter().filter(|f| !f.yanked).collect()
    }

    /// Get statistics
    pub fn stats(&self) -> IndexStats {
        let total = self.files.len();
        let wheels = self.get_wheels().len();
        let sdists = self.get_sdists().len();
        let yanked = self.files.iter().filter(|f| f.yanked).count();

        IndexStats {
            total,
            wheels,
            sdists,
            yanked,
        }
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total: usize,
    pub wheels: usize,
    pub sdists: usize,
    pub yanked: usize,
}

/// PEP 691 handler
pub struct Pep691Handler;

impl Pep691Handler {
    /// Scan directory for packages
    pub fn scan_directory(path: &Path) -> Result<FileIndex, String> {
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", path.display()));
        }

        let mut index = FileIndex::new(path.to_path_buf());

        for entry in std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                if let Some(filename) = entry_path.file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        // Check if it's a package file
                        if Self::is_package_file(filename_str) {
                            let file_info = FileUrlInfo::new(entry_path.clone(), filename_str.to_string());
                            index.add_file(file_info);
                        }
                    }
                }
            }
        }

        Ok(index)
    }

    /// Check if file is a package file
    fn is_package_file(filename: &str) -> bool {
        filename.ends_with(".whl")
            || filename.ends_with(".tar.gz")
            || filename.ends_with(".zip")
            || filename.ends_with(".tar.bz2")
            || filename.ends_with(".tar.xz")
            || filename.ends_with(".egg")
    }

    /// Generate PEP 691 JSON for index
    pub fn generate_json(index: &FileIndex) -> String {
        let mut files = vec![];

        for file in &index.files {
            let mut file_json = serde_json::json!({
                "filename": file.filename,
                "url": file.get_file_url(),
                "yanked": file.yanked,
            });

            // Add hashes if present
            if !file.hashes.is_empty() {
                let hashes_obj = serde_json::json!(file.hashes);
                file_json["hashes"] = hashes_obj;
            }

            // Add requires-python if present
            if let Some(req_python) = &file.requires_python {
                file_json["requires-python"] = serde_json::json!(req_python);
            }

            files.push(file_json);
        }

        serde_json::json!({
            "files": files,
        })
        .to_string()
    }

    /// Validate file URL
    pub fn validate_file_url(url: &str) -> Result<PathBuf, String> {
        if !url.starts_with("file://") {
            return Err("URL must start with file://".to_string());
        }

        let path_str = &url[7..]; // Remove "file://"
        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_url_info_new() {
        let path = PathBuf::from("/path/to/package.whl");
        let info = FileUrlInfo::new(path.clone(), "package.whl".to_string());
        assert_eq!(info.filename, "package.whl");
        assert!(!info.yanked);
    }

    #[test]
    fn test_file_url_info_add_hash() {
        let path = PathBuf::from("/path/to/package.whl");
        let mut info = FileUrlInfo::new(path, "package.whl".to_string());
        info.add_hash("sha256", "abc123");
        assert_eq!(info.hashes.get("sha256"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_file_url_info_set_requires_python() {
        let path = PathBuf::from("/path/to/package.whl");
        let mut info = FileUrlInfo::new(path, "package.whl".to_string());
        info.set_requires_python(">=3.8");
        assert_eq!(info.requires_python, Some(">=3.8".to_string()));
    }

    #[test]
    fn test_file_url_info_set_yanked() {
        let path = PathBuf::from("/path/to/package.whl");
        let mut info = FileUrlInfo::new(path, "package.whl".to_string());
        info.set_yanked(true);
        assert!(info.yanked);
    }

    #[test]
    fn test_file_url_info_get_file_url() {
        let path = PathBuf::from("/path/to/package.whl");
        let info = FileUrlInfo::new(path, "package.whl".to_string());
        let url = info.get_file_url();
        assert!(url.starts_with("file://"));
    }

    #[test]
    fn test_file_index_new() {
        let path = PathBuf::from("/path/to/index");
        let index = FileIndex::new(path.clone());
        assert_eq!(index.files.len(), 0);
    }

    #[test]
    fn test_file_index_add_file() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        let file_info = FileUrlInfo::new(
            PathBuf::from("/path/to/package.whl"),
            "package.whl".to_string(),
        );
        index.add_file(file_info);
        assert_eq!(index.files.len(), 1);
    }

    #[test]
    fn test_file_index_find_by_package() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        let file_info = FileUrlInfo::new(
            PathBuf::from("/path/to/requests.whl"),
            "requests.whl".to_string(),
        );
        index.add_file(file_info);
        let found = index.find_by_package("requests");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_file_index_get_wheels() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        index.add_file(FileUrlInfo::new(
            PathBuf::from("/path/to/package.whl"),
            "package.whl".to_string(),
        ));
        index.add_file(FileUrlInfo::new(
            PathBuf::from("/path/to/package.tar.gz"),
            "package.tar.gz".to_string(),
        ));
        assert_eq!(index.get_wheels().len(), 1);
    }

    #[test]
    fn test_file_index_get_sdists() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        index.add_file(FileUrlInfo::new(
            PathBuf::from("/path/to/package.tar.gz"),
            "package.tar.gz".to_string(),
        ));
        assert_eq!(index.get_sdists().len(), 1);
    }

    #[test]
    fn test_file_index_stats() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        index.add_file(FileUrlInfo::new(
            PathBuf::from("/path/to/package.whl"),
            "package.whl".to_string(),
        ));
        let stats = index.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.wheels, 1);
    }

    #[test]
    fn test_pep691_handler_is_package_file() {
        assert!(Pep691Handler::is_package_file("package.whl"));
        assert!(Pep691Handler::is_package_file("package.tar.gz"));
        assert!(Pep691Handler::is_package_file("package.zip"));
        assert!(!Pep691Handler::is_package_file("readme.txt"));
    }

    #[test]
    fn test_pep691_handler_validate_file_url() {
        let result = Pep691Handler::validate_file_url("file:///nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_pep691_handler_generate_json() {
        let path = PathBuf::from("/path/to/index");
        let mut index = FileIndex::new(path);
        index.add_file(FileUrlInfo::new(
            PathBuf::from("/path/to/package.whl"),
            "package.whl".to_string(),
        ));
        let json = Pep691Handler::generate_json(&index);
        assert!(json.contains("package.whl"));
    }
}
