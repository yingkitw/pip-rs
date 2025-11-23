/// Environment marker evaluation with platform support
/// 
/// This module evaluates PEP 508 environment markers with support for
/// platform overrides (--python-version, --platform, etc.)

use std::collections::HashMap;

/// Environment marker context
#[derive(Clone, Debug)]
pub struct EnvironmentContext {
    /// Python version (e.g., "3.11")
    pub python_version: String,
    /// Platform (e.g., "linux", "darwin", "win32")
    pub platform: String,
    /// Implementation (e.g., "cpython", "pypy")
    pub implementation: String,
    /// Architecture (e.g., "x86_64", "arm64")
    pub architecture: String,
    /// OS name (e.g., "posix", "nt")
    pub os_name: String,
    /// System (e.g., "Linux", "Darwin", "Windows")
    pub system: String,
}

impl EnvironmentContext {
    /// Create default environment context from current system
    pub fn default() -> Self {
        Self {
            python_version: Self::get_python_version(),
            platform: Self::get_platform(),
            implementation: Self::get_implementation(),
            architecture: Self::get_architecture(),
            os_name: Self::get_os_name(),
            system: Self::get_system(),
        }
    }

    /// Create environment context with overrides
    pub fn with_overrides(
        python_version: Option<String>,
        platform: Option<String>,
        implementation: Option<String>,
    ) -> Self {
        let mut ctx = Self::default();
        if let Some(version) = python_version {
            ctx.python_version = version;
        }
        if let Some(plat) = platform {
            ctx.platform = plat;
        }
        if let Some(impl_name) = implementation {
            ctx.implementation = impl_name;
        }
        ctx
    }

    /// Get Python version
    fn get_python_version() -> String {
        format!("{}.{}", 3, 11) // Default to 3.11
    }

    /// Get platform
    fn get_platform() -> String {
        if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "win32".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Get implementation
    fn get_implementation() -> String {
        "cpython".to_string()
    }

    /// Get architecture
    fn get_architecture() -> String {
        if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else if cfg!(target_arch = "x86") {
            "x86".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Get OS name
    fn get_os_name() -> String {
        if cfg!(unix) {
            "posix".to_string()
        } else if cfg!(windows) {
            "nt".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Get system
    fn get_system() -> String {
        if cfg!(target_os = "linux") {
            "Linux".to_string()
        } else if cfg!(target_os = "macos") {
            "Darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Convert to marker variables map
    pub fn to_marker_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("python_version".to_string(), self.python_version.clone());
        vars.insert("platform".to_string(), self.platform.clone());
        vars.insert("implementation_name".to_string(), self.implementation.clone());
        vars.insert("platform_machine".to_string(), self.architecture.clone());
        vars.insert("os_name".to_string(), self.os_name.clone());
        vars.insert("sys_platform".to_string(), self.platform.clone());
        vars
    }
}

/// Evaluate environment markers
pub struct MarkerEvaluator {
    context: EnvironmentContext,
}

impl MarkerEvaluator {
    pub fn new(context: EnvironmentContext) -> Self {
        Self { context }
    }

    /// Evaluate a marker expression
    pub fn evaluate(&self, marker: &str) -> bool {
        // Simple marker evaluation for common cases
        let marker = marker.trim();

        // Handle "and" and "or" operators
        if marker.contains(" and ") {
            return marker.split(" and ").all(|m| self.evaluate(m));
        }
        if marker.contains(" or ") {
            return marker.split(" or ").any(|m| self.evaluate(m));
        }

        // Handle comparison operators
        if let Some(pos) = marker.find("==") {
            let (left, right) = marker.split_at(pos);
            let right = &right[2..].trim();
            return self.compare_equal(left.trim(), right);
        }
        if let Some(pos) = marker.find("!=") {
            let (left, right) = marker.split_at(pos);
            let right = &right[2..].trim();
            return !self.compare_equal(left.trim(), right);
        }
        if let Some(pos) = marker.find(">=") {
            let (left, right) = marker.split_at(pos);
            let right = &right[2..].trim();
            return self.compare_gte(left.trim(), right);
        }
        if let Some(pos) = marker.find("<=") {
            let (left, right) = marker.split_at(pos);
            let right = &right[2..].trim();
            return self.compare_lte(left.trim(), right);
        }
        if let Some(pos) = marker.find('>') {
            let (left, right) = marker.split_at(pos);
            let right = &right[1..].trim();
            return self.compare_gt(left.trim(), right);
        }
        if let Some(pos) = marker.find('<') {
            let (left, right) = marker.split_at(pos);
            let right = &right[1..].trim();
            return self.compare_lt(left.trim(), right);
        }

        false
    }

    /// Get variable value
    fn get_var(&self, name: &str) -> Option<String> {
        let vars = self.context.to_marker_vars();
        vars.get(name).cloned()
    }

    /// Compare equal
    fn compare_equal(&self, left: &str, right: &str) -> bool {
        if let Some(left_val) = self.get_var(left) {
            let right_val = right.trim_matches('"').trim_matches('\'');
            return left_val == right_val;
        }
        false
    }

    /// Compare greater than or equal
    fn compare_gte(&self, left: &str, right: &str) -> bool {
        if let Some(left_val) = self.get_var(left) {
            let right_val = right.trim_matches('"').trim_matches('\'');
            return left_val.as_str() >= right_val;
        }
        false
    }

    /// Compare less than or equal
    fn compare_lte(&self, left: &str, right: &str) -> bool {
        if let Some(left_val) = self.get_var(left) {
            let right_val = right.trim_matches('"').trim_matches('\'');
            return left_val.as_str() <= right_val;
        }
        false
    }

    /// Compare greater than
    fn compare_gt(&self, left: &str, right: &str) -> bool {
        if let Some(left_val) = self.get_var(left) {
            let right_val = right.trim_matches('"').trim_matches('\'');
            return left_val.as_str() > right_val;
        }
        false
    }

    /// Compare less than
    fn compare_lt(&self, left: &str, right: &str) -> bool {
        if let Some(left_val) = self.get_var(left) {
            let right_val = right.trim_matches('"').trim_matches('\'');
            return left_val.as_str() < right_val;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_context_default() {
        let ctx = EnvironmentContext::default();
        assert!(!ctx.python_version.is_empty());
        assert!(!ctx.platform.is_empty());
        assert_eq!(ctx.implementation, "cpython");
    }

    #[test]
    fn test_environment_context_with_overrides() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.9".to_string()),
            Some("linux".to_string()),
            None,
        );
        assert_eq!(ctx.python_version, "3.9");
        assert_eq!(ctx.platform, "linux");
    }

    #[test]
    fn test_marker_evaluator_equal() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.11".to_string()),
            None,
            None,
        );
        let evaluator = MarkerEvaluator::new(ctx);
        assert!(evaluator.evaluate("python_version == \"3.11\""));
        assert!(!evaluator.evaluate("python_version == \"3.10\""));
    }

    #[test]
    fn test_marker_evaluator_not_equal() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.11".to_string()),
            None,
            None,
        );
        let evaluator = MarkerEvaluator::new(ctx);
        assert!(evaluator.evaluate("python_version != \"3.10\""));
        assert!(!evaluator.evaluate("python_version != \"3.11\""));
    }

    #[test]
    fn test_marker_evaluator_and() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.11".to_string()),
            Some("linux".to_string()),
            None,
        );
        let evaluator = MarkerEvaluator::new(ctx);
        assert!(evaluator.evaluate("python_version == \"3.11\" and platform == \"linux\""));
        assert!(!evaluator.evaluate("python_version == \"3.11\" and platform == \"darwin\""));
    }

    #[test]
    fn test_marker_evaluator_or() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.11".to_string()),
            None,
            None,
        );
        let evaluator = MarkerEvaluator::new(ctx);
        assert!(evaluator.evaluate("python_version == \"3.10\" or python_version == \"3.11\""));
        assert!(!evaluator.evaluate("python_version == \"3.9\" or python_version == \"3.10\""));
    }

    #[test]
    fn test_marker_evaluator_gte() {
        let ctx = EnvironmentContext::with_overrides(
            Some("3.11".to_string()),
            None,
            None,
        );
        let evaluator = MarkerEvaluator::new(ctx);
        assert!(evaluator.evaluate("python_version >= \"3.10\""));
        assert!(evaluator.evaluate("python_version >= \"3.11\""));
        assert!(!evaluator.evaluate("python_version >= \"3.12\""));
    }
}
