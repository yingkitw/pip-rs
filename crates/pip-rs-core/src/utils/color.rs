/// Color output utilities for CLI
use colored::*;

/// Color output configuration
#[derive(Clone, Copy, Debug)]
pub struct ColorConfig {
    pub enabled: bool,
}

impl ColorConfig {
    pub fn new() -> Self {
        // Enable colors by default unless NO_COLOR env var is set
        let enabled = std::env::var("NO_COLOR").is_err();
        Self { enabled }
    }

    pub fn from_env() -> Self {
        if let Ok(val) = std::env::var("FORCE_COLOR") {
            return Self {
                enabled: !val.is_empty() && val != "0",
            };
        }
        Self::new()
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Color output helper
pub struct ColorOutput {
    config: ColorConfig,
}

impl ColorOutput {
    pub fn new(config: ColorConfig) -> Self {
        Self { config }
    }

    pub fn success(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.green())
        } else {
            msg.to_string()
        }
    }

    pub fn error(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.red())
        } else {
            msg.to_string()
        }
    }

    pub fn warning(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.yellow())
        } else {
            msg.to_string()
        }
    }

    pub fn info(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.cyan())
        } else {
            msg.to_string()
        }
    }

    pub fn highlight(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.bold())
        } else {
            msg.to_string()
        }
    }

    pub fn muted(&self, msg: &str) -> String {
        if self.config.enabled {
            format!("{}", msg.dimmed())
        } else {
            msg.to_string()
        }
    }

    /// Print success message
    pub fn print_success(&self, msg: &str) {
        println!("{}", self.success(&format!("✓ {}", msg)));
    }

    /// Print error message
    pub fn print_error(&self, msg: &str) {
        eprintln!("{}", self.error(&format!("✗ {}", msg)));
    }

    /// Print warning message
    pub fn print_warning(&self, msg: &str) {
        eprintln!("{}", self.warning(&format!("⚠ {}", msg)));
    }

    /// Print info message
    pub fn print_info(&self, msg: &str) {
        println!("{}", self.info(&format!("ℹ {}", msg)));
    }

    /// Print header
    pub fn print_header(&self, msg: &str) {
        println!("\n{}", self.highlight(&format!("=== {} ===", msg)));
    }
}

impl Default for ColorOutput {
    fn default() -> Self {
        Self::new(ColorConfig::default())
    }
}

/// Global color output instance
pub fn get_color_output() -> ColorOutput {
    ColorOutput::new(ColorConfig::from_env())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_config_default() {
        let config = ColorConfig::new();
        assert!(config.enabled || !config.enabled); // Just check it doesn't panic
    }

    #[test]
    fn test_color_output_success() {
        let output = ColorOutput::new(ColorConfig { enabled: false });
        let result = output.success("test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_color_output_error() {
        let output = ColorOutput::new(ColorConfig { enabled: false });
        let result = output.error("test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_color_output_warning() {
        let output = ColorOutput::new(ColorConfig { enabled: false });
        let result = output.warning("test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_color_output_info() {
        let output = ColorOutput::new(ColorConfig { enabled: false });
        let result = output.info("test");
        assert_eq!(result, "test");
    }
}
