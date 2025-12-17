/// Error handling and diagnostics
use std::fmt;

#[derive(Debug)]
pub enum PipError {
    /// Network error with retry information
    NetworkError {
        message: String,
        retries: u32,
        last_error: String,
    },
    /// Package not found on PyPI
    PackageNotFound {
        name: String,
        version: Option<String>,
    },
    /// Dependency conflict
    DependencyConflict {
        package: String,
        required: String,
        installed: String,
    },
    /// Invalid requirement specification
    InvalidRequirement {
        spec: String,
        reason: String,
    },
    /// Installation failed
    InstallationFailed {
        package: String,
        reason: String,
    },
    /// Uninstallation failed
    UninstallationFailed {
        package: String,
        reason: String,
    },
    /// File system error
    FileSystemError {
        path: String,
        operation: String,
        reason: String,
    },
    /// Configuration error
    ConfigError {
        message: String,
    },
    /// Dependency resolution error
    DependencyResolutionError {
        package: String,
        reason: String,
    },
    /// Invalid package error
    InvalidPackage {
        name: String,
        reason: String,
    },
}

impl fmt::Display for PipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipError::NetworkError {
                message,
                retries,
                last_error,
            } => {
                write!(
                    f,
                    "Network error: {} (failed after {} retries)\nLast error: {}",
                    message, retries, last_error
                )
            }
            PipError::PackageNotFound { name, version } => {
                if let Some(v) = version {
                    write!(f, "Package not found: {} version {}", name, v)
                } else {
                    write!(f, "Package not found: {}", name)
                }
            }
            PipError::DependencyConflict {
                package,
                required,
                installed,
            } => {
                write!(
                    f,
                    "Dependency conflict: {} requires {} but {} is installed",
                    package, required, installed
                )
            }
            PipError::InvalidRequirement { spec, reason } => {
                write!(f, "Invalid requirement '{}': {}", spec, reason)
            }
            PipError::InstallationFailed { package, reason } => {
                write!(f, "Failed to install {}: {}", package, reason)
            }
            PipError::UninstallationFailed { package, reason } => {
                write!(f, "Failed to uninstall {}: {}", package, reason)
            }
            PipError::FileSystemError {
                path,
                operation,
                reason,
            } => {
                write!(
                    f,
                    "File system error during {}: {} ({})",
                    operation, path, reason
                )
            }
            PipError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            PipError::DependencyResolutionError { package, reason } => {
                write!(f, "Failed to resolve dependencies for {}: {}", package, reason)
            }
            PipError::InvalidPackage { name, reason } => {
                write!(f, "Invalid package {}: {}", name, reason)
            }
        }
    }
}

impl std::error::Error for PipError {}

/// Helper functions for better error messages
pub fn suggest_fix(error: &str) -> Option<String> {
    if error.contains("No such file or directory") {
        Some("Check that the file path is correct and the file exists".to_string())
    } else if error.contains("Permission denied") {
        Some("You may need to run with elevated privileges or check file permissions".to_string())
    } else if error.contains("Connection refused") {
        Some("Check your internet connection and PyPI server availability".to_string())
    } else if error.contains("Timeout") || error.contains("timed out") {
        Some("The request timed out. Try again or check your network connection. For large packages, consider using --timeout to increase the timeout".to_string())
    } else if error.contains("DNS") {
        Some("DNS resolution failed. Check your internet connection".to_string())
    } else if error.contains("404") || error.contains("not found") {
        Some("The requested resource was not found. Check the package name and version. Try: pip search <package-name>".to_string())
    } else if error.contains("403") {
        Some("Access denied. You may need authentication credentials or check if the package requires special access".to_string())
    } else if error.contains("500") || error.contains("502") || error.contains("503") {
        Some("PyPI server error. Try again later or use --retries to increase retry attempts".to_string())
    } else if error.contains("SSL") || error.contains("certificate") || error.contains("TLS") {
        Some("SSL certificate error. If this is a trusted host, use --trusted-host <hostname>".to_string())
    } else if error.contains("Invalid requirement") {
        Some("Check the requirement format. Use 'package==version' or 'package>=version'. See PEP 508 for details".to_string())
    } else if error.contains("Dependency conflict") {
        Some("Try upgrading conflicting packages or use --no-deps to skip dependency checks".to_string())
    } else if error.contains("Failed to parse JSON") {
        Some("The response from PyPI was invalid. This may be a temporary issue. Try again later".to_string())
    } else {
        None
    }
}

/// Format error with suggestions
pub fn format_error_with_suggestion(error: &str) -> String {
    let mut msg = format!("ERROR: {}", error);
    if let Some(suggestion) = suggest_fix(error) {
        msg.push_str(&format!("\nSuggestion: {}", suggestion));
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_display() {
        let err = PipError::NetworkError {
            message: "Connection failed".to_string(),
            retries: 3,
            last_error: "timeout".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Network error"));
        assert!(msg.contains("3 retries"));
    }

    #[test]
    fn test_package_not_found_display() {
        let err = PipError::PackageNotFound {
            name: "nonexistent".to_string(),
            version: Some("1.0.0".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("nonexistent"));
        assert!(msg.contains("1.0.0"));
    }

    #[test]
    fn test_suggest_fix() {
        assert!(suggest_fix("Connection refused").is_some());
        assert!(suggest_fix("Timeout").is_some());
        assert!(suggest_fix("404 Not Found").is_some());
    }
}
