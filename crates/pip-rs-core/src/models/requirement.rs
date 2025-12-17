/// Requirement specification and parsing
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub name: String,
    pub specs: Vec<VersionSpec>,
    pub extras: Vec<String>,
    pub marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionSpec {
    pub op: VersionOp,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Compatible,
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Split on semicolon for marker
        let (req_part, marker) = if let Some(idx) = s.find(';') {
            let (req, marker) = s.split_at(idx);
            (req.trim(), Some(marker[1..].trim().to_string()))
        } else {
            (s, None)
        };

        // Parse package name
        let mut name = String::new();
        let mut pos = 0;
        let req_bytes = req_part.as_bytes();

        for (i, &ch) in req_bytes.iter().enumerate() {
            let c = ch as char;
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(c);
                pos = i + 1;
            } else {
                break;
            }
        }

        // Parse extras and version specs from the remainder
        let remainder = &req_part[pos..];
        let mut extras = Vec::new();
        let mut specs = Vec::new();
        let mut spec_start = 0;

        if remainder.starts_with('[') {
            if let Some(bracket_end) = remainder.find(']') {
                let extras_str = &remainder[1..bracket_end];
                extras = extras_str.split(',').map(|e| e.trim().to_string()).collect();
                spec_start = bracket_end + 1;
            }
        }

        let version_part = remainder[spec_start..].trim();
        if !version_part.is_empty() {
            specs = parse_version_specs(version_part)?;
        }

        Ok(Requirement {
            name: name.to_lowercase().replace('_', "-"),
            specs,
            extras,
            marker,
        })
    }
}

fn parse_version_specs(s: &str) -> Result<Vec<VersionSpec>, String> {
    let mut specs = Vec::new();
    let s = s.trim();
    let mut pos = 0;

    while pos < s.len() {
        let remaining = &s[pos..].trim_start();
        if remaining.is_empty() {
            break;
        }

        let (op, skip) = if remaining.starts_with("==") {
            (VersionOp::Eq, 2)
        } else if remaining.starts_with("!=") {
            (VersionOp::NotEq, 2)
        } else if remaining.starts_with("<=") {
            (VersionOp::LtEq, 2)
        } else if remaining.starts_with(">=") {
            (VersionOp::GtEq, 2)
        } else if remaining.starts_with("<") {
            (VersionOp::Lt, 1)
        } else if remaining.starts_with(">") {
            (VersionOp::Gt, 1)
        } else if remaining.starts_with("~=") {
            (VersionOp::Compatible, 2)
        } else {
            return Err(format!("Invalid version spec: {}", remaining));
        };

        let rest = &remaining[skip..].trim_start();
        let mut version = String::new();
        let mut version_end = 0;

        for (i, ch) in rest.chars().enumerate() {
            if ch.is_alphanumeric() || ch == '.' || ch == '*' || ch == '+' {
                version.push(ch);
                version_end = i + 1;
            } else if ch == ',' {
                break;
            } else {
                version_end = i + 1;
            }
        }

        if version.is_empty() {
            return Err("Empty version in spec".to_string());
        }

        specs.push(VersionSpec { op, version });
        pos += s[pos..].len() - rest.len() + version_end;
        if pos < s.len() && s[pos..].trim_start().starts_with(',') {
            pos += s[pos..].find(',').unwrap_or(0) + 1;
        }
    }

    Ok(specs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_requirement() {
        let req: Requirement = "requests".parse().unwrap();
        assert_eq!(req.name, "requests");
        assert!(req.specs.is_empty());
    }

    #[test]
    fn test_parse_requirement_with_version() {
        let req: Requirement = "requests>=2.28.0".parse().unwrap();
        assert_eq!(req.name, "requests");
        assert_eq!(req.specs.len(), 1);
        assert_eq!(req.specs[0].op, VersionOp::GtEq);
        assert_eq!(req.specs[0].version, "2.28.0");
    }

    #[test]
    fn test_parse_requirement_with_extras() {
        let req: Requirement = "requests[security]>=2.28.0".parse().unwrap();
        assert_eq!(req.name, "requests");
        assert_eq!(req.extras, vec!["security"]);
    }
}
