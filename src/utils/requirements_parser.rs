/// Requirements file parsing with continuation support
/// 
/// This module handles parsing requirements files with proper support for
/// line continuations (backslash at end of line).

use std::path::Path;

/// Parsed requirement line
#[derive(Clone, Debug)]
pub struct ParsedRequirement {
    pub requirement: String,
    pub line_number: usize,
    pub is_editable: bool,
    pub is_comment: bool,
}

/// Requirements file parser
pub struct RequirementsParser;

impl RequirementsParser {
    /// Parse requirements file with continuation support
    pub fn parse_file(path: &Path) -> Result<Vec<ParsedRequirement>, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read requirements file: {}", e))?;

        Ok(Self::parse_content(&content))
    }

    /// Parse requirements content with continuation support
    pub fn parse_content(content: &str) -> Vec<ParsedRequirement> {
        let mut requirements = vec![];
        let mut current_line = String::new();
        let mut line_number = 0;
        let mut start_line = 0;

        for line in content.lines() {
            line_number += 1;
            let trimmed = line.trim_end();

            // Handle line continuation (backslash at end)
            if trimmed.ends_with('\\') {
                // Remove the backslash and trailing whitespace
                let continued = trimmed[..trimmed.len() - 1].trim_end();
                current_line.push_str(continued);
                // Add space if not empty to separate from next line
                if !current_line.is_empty() && !current_line.ends_with(' ') {
                    current_line.push(' ');
                }
                continue;
            }

            // Complete the line
            current_line.push_str(trimmed);

            if start_line == 0 {
                start_line = line_number;
            }

            // Process the complete line
            if !current_line.is_empty() {
                if let Some(req) = Self::parse_line(&current_line, start_line) {
                    requirements.push(req);
                }
            }

            // Reset for next line
            current_line.clear();
            start_line = 0;
        }

        // Handle any remaining content
        if !current_line.is_empty() {
            if let Some(req) = Self::parse_line(&current_line, start_line) {
                requirements.push(req);
            }
        }

        requirements
    }

    /// Parse a single requirement line
    fn parse_line(line: &str, line_number: usize) -> Option<ParsedRequirement> {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            return None;
        }

        // Handle comments
        if trimmed.starts_with('#') {
            return Some(ParsedRequirement {
                requirement: trimmed.to_string(),
                line_number,
                is_editable: false,
                is_comment: true,
            });
        }

        // Handle editable installs
        let (is_editable, requirement) = if trimmed.starts_with("-e ") {
            (true, trimmed[3..].trim())
        } else {
            (false, trimmed)
        };

        // Skip other flags
        if requirement.starts_with('-') {
            return None;
        }

        Some(ParsedRequirement {
            requirement: requirement.to_string(),
            line_number,
            is_editable,
            is_comment: false,
        })
    }

    /// Validate continuation handling
    pub fn validate_continuations(content: &str) -> Result<(), String> {
        let mut in_continuation = false;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_end();

            if trimmed.ends_with('\\') {
                in_continuation = true;
                // Check for space after backslash (invalid)
                if trimmed.len() > 1 && trimmed.chars().rev().nth(1) == Some(' ') {
                    return Err(format!(
                        "Line {}: Space before backslash continuation",
                        i + 1
                    ));
                }
            } else if in_continuation {
                in_continuation = false;
            }
        }

        if in_continuation {
            return Err("File ends with continuation backslash".to_string());
        }

        Ok(())
    }

    /// Get continuation statistics
    pub fn get_stats(content: &str) -> ContinuationStats {
        let mut total_lines = 0;
        let mut continuation_lines = 0;
        let mut logical_lines = 0;
        let mut in_continuation = false;

        for line in content.lines() {
            total_lines += 1;
            let trimmed = line.trim_end();

            if trimmed.ends_with('\\') {
                continuation_lines += 1;
                in_continuation = true;
            } else {
                if !in_continuation {
                    logical_lines += 1;
                } else {
                    logical_lines += 1;
                    in_continuation = false;
                }
            }
        }

        ContinuationStats {
            total_lines,
            continuation_lines,
            logical_lines,
        }
    }
}

/// Continuation statistics
#[derive(Debug, Clone)]
pub struct ContinuationStats {
    pub total_lines: usize,
    pub continuation_lines: usize,
    pub logical_lines: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_requirement() {
        let content = "requests==2.28.0\n";
        let reqs = RequirementsParser::parse_content(content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].requirement, "requests==2.28.0");
        assert!(!reqs[0].is_editable);
    }

    #[test]
    fn test_parse_editable_requirement() {
        let content = "-e git+https://github.com/user/repo.git\n";
        let reqs = RequirementsParser::parse_content(content);
        assert_eq!(reqs.len(), 1);
        assert!(reqs[0].is_editable);
    }

    #[test]
    fn test_parse_comment() {
        let content = "# This is a comment\n";
        let reqs = RequirementsParser::parse_content(content);
        assert_eq!(reqs.len(), 1);
        assert!(reqs[0].is_comment);
    }

    #[test]
    fn test_parse_continuation_simple() {
        let content = "requests==2.28.0 \\\n    # with comment\n";
        let reqs = RequirementsParser::parse_content(content);
        assert!(!reqs.is_empty());
    }

    #[test]
    fn test_parse_continuation_multiple() {
        let content = "package1==1.0.0 \\\n    package2==2.0.0 \\\n    package3==3.0.0\n";
        let reqs = RequirementsParser::parse_content(content);
        assert!(!reqs.is_empty());
    }

    #[test]
    fn test_parse_empty_lines() {
        let content = "requests==2.28.0\n\n\ndjango==3.2.0\n";
        let reqs = RequirementsParser::parse_content(content);
        assert_eq!(reqs.len(), 2);
    }

    #[test]
    fn test_validate_continuations_valid() {
        let content = "requests==2.28.0\\\ndjango==3.2.0\n";
        assert!(RequirementsParser::validate_continuations(content).is_ok());
    }

    #[test]
    fn test_validate_continuations_invalid_end() {
        let content = "requests==2.28.0 \\\n";
        assert!(RequirementsParser::validate_continuations(content).is_err());
    }

    #[test]
    fn test_get_stats() {
        let content = "requests==2.28.0 \\\n    django==3.2.0\n";
        let stats = RequirementsParser::get_stats(content);
        assert_eq!(stats.total_lines, 2);
        assert_eq!(stats.continuation_lines, 1);
    }

    #[test]
    fn test_parse_multiple_requirements() {
        let content = "requests==2.28.0\ndjango==3.2.0\nflask==2.0.0\n";
        let reqs = RequirementsParser::parse_content(content);
        assert_eq!(reqs.len(), 3);
    }

    #[test]
    fn test_parse_with_flags() {
        let content = "--index-url https://pypi.org/simple\nrequests==2.28.0\n";
        let reqs = RequirementsParser::parse_content(content);
        // Flags should be skipped
        assert_eq!(reqs.len(), 1);
    }

    #[test]
    fn test_parse_continuation_preserves_content() {
        let content = "requests==2.28.0\\\n# continuation\n";
        let reqs = RequirementsParser::parse_content(content);
        assert!(!reqs.is_empty());
        // Should have parsed the requirement
        assert!(reqs[0].requirement.contains("requests"));
    }
}
