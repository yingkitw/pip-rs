/// Input validation and error handling utilities
use anyhow::{Result, anyhow};

/// Validate package name
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Package name cannot be empty"));
    }

    if name.len() > 255 {
        return Err(anyhow!("Package name too long (max 255 characters)"));
    }

    // Check for invalid characters
    if !name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '*'
    }) {
        return Err(anyhow!(
            "Invalid package name '{}'. Only alphanumeric, dash, underscore, and dot allowed",
            name
        ));
    }

    Ok(())
}

/// Validate version specification
pub fn validate_version_spec(spec: &str) -> Result<()> {
    if spec.is_empty() {
        return Err(anyhow!("Version specification cannot be empty"));
    }

    // Check for valid operators
    let valid_operators = vec!["==", "!=", "<=", ">=", "<", ">", "~=", "==="];
    let has_valid_op = valid_operators.iter().any(|op| spec.contains(op));

    if !has_valid_op && !spec.chars().all(|c| c.is_numeric() || c == '.') {
        return Err(anyhow!(
            "Invalid version specification '{}'. Use operators like ==, >=, <, etc.",
            spec
        ));
    }

    Ok(())
}

/// Validate URL
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!(
            "Invalid URL '{}'. Must start with http:// or https://",
            url
        ));
    }

    if url.len() > 2048 {
        return Err(anyhow!("URL too long (max 2048 characters)"));
    }

    Ok(())
}

/// Validate file path
pub fn validate_file_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("File path cannot be empty"));
    }

    if path.len() > 4096 {
        return Err(anyhow!("File path too long (max 4096 characters)"));
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(anyhow!("File path contains null bytes"));
    }

    Ok(())
}

/// Validate requirements file
pub fn validate_requirements_file(content: &str) -> Result<Vec<String>> {
    let mut requirements = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for invalid characters
        if trimmed.contains('\0') {
            return Err(anyhow!(
                "Line {}: Contains null bytes",
                line_num + 1
            ));
        }

        // Check line length
        if trimmed.len() > 1024 {
            return Err(anyhow!(
                "Line {}: Too long (max 1024 characters)",
                line_num + 1
            ));
        }

        requirements.push(trimmed.to_string());
    }

    if requirements.is_empty() {
        return Err(anyhow!("Requirements file is empty or contains only comments"));
    }

    Ok(requirements)
}

/// Validate Python version string
pub fn validate_python_version(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(anyhow!("Python version cannot be empty"));
    }

    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return Err(anyhow!(
            "Invalid Python version '{}'. Expected format: X.Y or X.Y.Z",
            version
        ));
    }

    for part in parts {
        if part.is_empty() {
            return Err(anyhow!(
                "Invalid Python version '{}'. Empty version component",
                version
            ));
        }

        if !part.chars().all(|c| c.is_numeric()) {
            return Err(anyhow!(
                "Invalid Python version '{}'. Version components must be numeric",
                version
            ));
        }
    }

    Ok(())
}

/// Validate environment variable name
pub fn validate_env_var_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Environment variable name cannot be empty"));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow!(
            "Invalid environment variable name '{}'. Only alphanumeric and underscore allowed",
            name
        ));
    }

    if name.starts_with(|c: char| c.is_numeric()) {
        return Err(anyhow!(
            "Invalid environment variable name '{}'. Cannot start with a number",
            name
        ));
    }

    Ok(())
}

/// Sanitize user input
pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control() && *c != '\0')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name_valid() {
        assert!(validate_package_name("requests").is_ok());
        assert!(validate_package_name("my-package").is_ok());
        assert!(validate_package_name("my_package").is_ok());
        assert!(validate_package_name("my.package").is_ok());
    }

    #[test]
    fn test_validate_package_name_invalid() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("my package").is_err());
        assert!(validate_package_name("my@package").is_err());
    }

    #[test]
    fn test_validate_version_spec_valid() {
        assert!(validate_version_spec("==1.0.0").is_ok());
        assert!(validate_version_spec(">=1.0").is_ok());
        assert!(validate_version_spec("<2.0").is_ok());
        assert!(validate_version_spec("1.0.0").is_ok());
    }

    #[test]
    fn test_validate_version_spec_invalid() {
        assert!(validate_version_spec("").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://pypi.org/simple/").is_ok());
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("example.com").is_err());
    }

    #[test]
    fn test_validate_file_path_valid() {
        assert!(validate_file_path("/path/to/file.txt").is_ok());
        assert!(validate_file_path("./relative/path").is_ok());
    }

    #[test]
    fn test_validate_file_path_invalid() {
        assert!(validate_file_path("").is_err());
        assert!(validate_file_path("path\0with\0null").is_err());
    }

    #[test]
    fn test_validate_python_version_valid() {
        assert!(validate_python_version("3.11").is_ok());
        assert!(validate_python_version("3.11.0").is_ok());
        assert!(validate_python_version("2.7").is_ok());
    }

    #[test]
    fn test_validate_python_version_invalid() {
        assert!(validate_python_version("").is_err());
        assert!(validate_python_version("3").is_err());
        assert!(validate_python_version("3.x").is_err());
    }

    #[test]
    fn test_validate_env_var_name_valid() {
        assert!(validate_env_var_name("PATH").is_ok());
        assert!(validate_env_var_name("MY_VAR").is_ok());
        assert!(validate_env_var_name("VAR123").is_ok());
    }

    #[test]
    fn test_validate_env_var_name_invalid() {
        assert!(validate_env_var_name("").is_err());
        assert!(validate_env_var_name("123VAR").is_err());
        assert!(validate_env_var_name("MY-VAR").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("  hello  "), "hello");
        assert_eq!(sanitize_input("hello\0world"), "helloworld");
    }
}
