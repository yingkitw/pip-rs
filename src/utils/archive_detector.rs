/// Archive format detection and handling
/// 
/// This module detects and handles various archive formats used in Python packages.

use std::path::Path;

/// Supported archive formats
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP archive (.zip, .whl)
    Zip,
    /// TAR archive (.tar)
    Tar,
    /// GZIP compressed TAR (.tar.gz, .tgz)
    TarGz,
    /// BZIP2 compressed TAR (.tar.bz2, .tbz2)
    TarBz2,
    /// XZ compressed TAR (.tar.xz, .txz)
    TarXz,
    /// RAR archive (.rar)
    Rar,
    /// 7-Zip archive (.7z)
    SevenZ,
    /// Unknown format
    Unknown,
}

impl ArchiveFormat {
    /// Detect archive format from file extension
    pub fn from_extension(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();

        // Check for double extensions first
        if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
            return ArchiveFormat::TarGz;
        }
        if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") {
            return ArchiveFormat::TarBz2;
        }
        if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
            return ArchiveFormat::TarXz;
        }

        // Check single extensions
        if path_str.ends_with(".zip") || path_str.ends_with(".whl") {
            return ArchiveFormat::Zip;
        }
        if path_str.ends_with(".tar") {
            return ArchiveFormat::Tar;
        }
        if path_str.ends_with(".rar") {
            return ArchiveFormat::Rar;
        }
        if path_str.ends_with(".7z") {
            return ArchiveFormat::SevenZ;
        }

        ArchiveFormat::Unknown
    }

    /// Detect archive format from magic bytes
    pub fn from_magic_bytes(data: &[u8]) -> Self {
        if data.len() < 4 {
            return ArchiveFormat::Unknown;
        }

        // ZIP/WHEEL: 0x504B0304 (PK..)
        if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
            return ArchiveFormat::Zip;
        }

        // GZIP: 0x1F8B
        if data.len() >= 2 && data[0] == 0x1F && data[1] == 0x8B {
            return ArchiveFormat::TarGz;
        }

        // BZIP2: 0x425A (BZ)
        if data.len() >= 2 && data[0] == 0x42 && data[1] == 0x5A {
            return ArchiveFormat::TarBz2;
        }

        // XZ: 0xFD377A585A00 (FD 7zXZ\0)
        if data.len() >= 6
            && data[0] == 0xFD
            && data[1] == 0x37
            && data[2] == 0x7A
            && data[3] == 0x58
            && data[4] == 0x5A
            && data[5] == 0x00
        {
            return ArchiveFormat::TarXz;
        }

        // RAR: 0x526172211A0700 (Rar!\x1A\x07\x00)
        if data.len() >= 7
            && data[0] == 0x52
            && data[1] == 0x61
            && data[2] == 0x72
            && data[3] == 0x21
        {
            return ArchiveFormat::Rar;
        }

        // 7-Zip: 0x377ABCAF271C (7z¼¯'\x0C)
        if data.len() >= 6
            && data[0] == 0x37
            && data[1] == 0x7A
            && data[2] == 0xBC
            && data[3] == 0xAF
            && data[4] == 0x27
            && data[5] == 0x1C
        {
            return ArchiveFormat::SevenZ;
        }

        // TAR: Check for tar header signature at offset 257
        if data.len() >= 262 {
            let tar_sig = &data[257..262];
            if tar_sig == b"ustar" {
                return ArchiveFormat::Tar;
            }
        }

        ArchiveFormat::Unknown
    }

    /// Get file extension for format
    pub fn extension(&self) -> &str {
        match self {
            ArchiveFormat::Zip => ".zip",
            ArchiveFormat::Tar => ".tar",
            ArchiveFormat::TarGz => ".tar.gz",
            ArchiveFormat::TarBz2 => ".tar.bz2",
            ArchiveFormat::TarXz => ".tar.xz",
            ArchiveFormat::Rar => ".rar",
            ArchiveFormat::SevenZ => ".7z",
            ArchiveFormat::Unknown => ".unknown",
        }
    }

    /// Get human-readable format name
    pub fn name(&self) -> &str {
        match self {
            ArchiveFormat::Zip => "ZIP",
            ArchiveFormat::Tar => "TAR",
            ArchiveFormat::TarGz => "TAR+GZIP",
            ArchiveFormat::TarBz2 => "TAR+BZIP2",
            ArchiveFormat::TarXz => "TAR+XZ",
            ArchiveFormat::Rar => "RAR",
            ArchiveFormat::SevenZ => "7-Zip",
            ArchiveFormat::Unknown => "Unknown",
        }
    }

    /// Check if format is supported
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            ArchiveFormat::Zip | ArchiveFormat::TarGz | ArchiveFormat::TarBz2
        )
    }
}

/// Archive detector
pub struct ArchiveDetector;

impl ArchiveDetector {
    /// Detect archive format from file
    pub fn detect(path: &Path) -> ArchiveFormat {
        // Try magic bytes first
        if let Ok(data) = std::fs::read(path) {
            let format = ArchiveFormat::from_magic_bytes(&data);
            if format != ArchiveFormat::Unknown {
                return format;
            }
        }

        // Fall back to extension
        ArchiveFormat::from_extension(path)
    }

    /// Get error message for unsupported format
    pub fn unsupported_error(format: &ArchiveFormat) -> String {
        format!(
            "Unsupported archive format: {} ({})",
            format.name(),
            format.extension()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_from_extension_zip() {
        let path = Path::new("package.zip");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::Zip);
    }

    #[test]
    fn test_archive_format_from_extension_wheel() {
        let path = Path::new("package.whl");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::Zip);
    }

    #[test]
    fn test_archive_format_from_extension_tar_gz() {
        let path = Path::new("package.tar.gz");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::TarGz);
    }

    #[test]
    fn test_archive_format_from_extension_tgz() {
        let path = Path::new("package.tgz");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::TarGz);
    }

    #[test]
    fn test_archive_format_from_extension_tar_bz2() {
        let path = Path::new("package.tar.bz2");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::TarBz2);
    }

    #[test]
    fn test_archive_format_from_extension_tar() {
        let path = Path::new("package.tar");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::Tar);
    }

    #[test]
    fn test_archive_format_from_extension_unknown() {
        let path = Path::new("package.xyz");
        assert_eq!(ArchiveFormat::from_extension(path), ArchiveFormat::Unknown);
    }

    #[test]
    fn test_archive_format_extension() {
        assert_eq!(ArchiveFormat::Zip.extension(), ".zip");
        assert_eq!(ArchiveFormat::TarGz.extension(), ".tar.gz");
        assert_eq!(ArchiveFormat::TarBz2.extension(), ".tar.bz2");
    }

    #[test]
    fn test_archive_format_name() {
        assert_eq!(ArchiveFormat::Zip.name(), "ZIP");
        assert_eq!(ArchiveFormat::TarGz.name(), "TAR+GZIP");
        assert_eq!(ArchiveFormat::TarBz2.name(), "TAR+BZIP2");
    }

    #[test]
    fn test_archive_format_is_supported() {
        assert!(ArchiveFormat::Zip.is_supported());
        assert!(ArchiveFormat::TarGz.is_supported());
        assert!(ArchiveFormat::TarBz2.is_supported());
        assert!(!ArchiveFormat::Rar.is_supported());
        assert!(!ArchiveFormat::Unknown.is_supported());
    }

    #[test]
    fn test_archive_detector_unsupported_error() {
        let error = ArchiveDetector::unsupported_error(&ArchiveFormat::Rar);
        assert!(error.contains("RAR"));
        assert!(error.contains(".rar"));
    }

    #[test]
    fn test_archive_format_magic_bytes_zip() {
        let zip_magic = vec![0x50, 0x4B, 0x03, 0x04];
        assert_eq!(ArchiveFormat::from_magic_bytes(&zip_magic), ArchiveFormat::Zip);
    }

    #[test]
    fn test_archive_format_magic_bytes_gzip() {
        let mut gzip_magic = vec![0x1F, 0x8B];
        gzip_magic.extend_from_slice(&[0x00; 10]); // Add padding
        assert_eq!(
            ArchiveFormat::from_magic_bytes(&gzip_magic),
            ArchiveFormat::TarGz
        );
    }

    #[test]
    fn test_archive_format_magic_bytes_bzip2() {
        let mut bzip2_magic = vec![0x42, 0x5A];
        bzip2_magic.extend_from_slice(&[0x00; 10]); // Add padding
        assert_eq!(
            ArchiveFormat::from_magic_bytes(&bzip2_magic),
            ArchiveFormat::TarBz2
        );
    }
}
