/// PEP 508 Environment Markers
/// Handles parsing and evaluation of environment markers for conditional dependencies

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerCondition {
    pub variable: String,
    pub op: MarkerOp,
    pub value: String,
}

/// Environment variables for marker evaluation
#[derive(Debug, Clone)]
pub struct Environment {
    pub python_version: String,
    pub python_full_version: String,
    pub os_name: String,
    pub sys_platform: String,
    pub platform_release: String,
    pub platform_system: String,
    pub platform_version: String,
    pub platform_machine: String,
    pub platform_python_implementation: String,
    pub implementation_name: String,
    pub implementation_version: String,
}

impl Environment {
    /// Create environment from current system
    pub fn current() -> Self {
        let python_version = format!("{}.{}", 3, 11); // Default to 3.11
        let python_full_version = format!("{}.0", python_version);
        
        #[cfg(target_os = "macos")]
        let os_name = "posix";
        #[cfg(target_os = "linux")]
        let os_name = "posix";
        #[cfg(target_os = "windows")]
        let os_name = "nt";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let os_name = "posix";

        #[cfg(target_os = "macos")]
        let sys_platform = "darwin";
        #[cfg(target_os = "linux")]
        let sys_platform = "linux";
        #[cfg(target_os = "windows")]
        let sys_platform = "win32";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let sys_platform = "unknown";

        #[cfg(target_arch = "x86_64")]
        let platform_machine = "x86_64";
        #[cfg(target_arch = "aarch64")]
        let platform_machine = "arm64";
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        let platform_machine = "unknown";

        Self {
            python_version: python_version.clone(),
            python_full_version,
            os_name: os_name.to_string(),
            sys_platform: sys_platform.to_string(),
            platform_release: "unknown".to_string(),
            platform_system: "Darwin".to_string(),
            platform_version: "unknown".to_string(),
            platform_machine: platform_machine.to_string(),
            platform_python_implementation: "CPython".to_string(),
            implementation_name: "cpython".to_string(),
            implementation_version: python_version,
        }
    }
}

impl Marker {
    /// Parse a marker string
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("Empty marker".to_string());
        }
        Ok(Marker {
            expression: s.to_string(),
        })
    }

    /// Evaluate marker against environment
    pub fn evaluate(&self, env: &Environment) -> bool {
        self.evaluate_expression(&self.expression, env)
    }

    fn evaluate_expression(&self, expr: &str, env: &Environment) -> bool {
        // Handle 'and' and 'or' operators
        if let Some(idx) = expr.find(" or ") {
            let left = &expr[..idx];
            let right = &expr[idx + 4..];
            return self.evaluate_expression(left, env) || self.evaluate_expression(right, env);
        }

        if let Some(idx) = expr.find(" and ") {
            let left = &expr[..idx];
            let right = &expr[idx + 5..];
            return self.evaluate_expression(left, env) && self.evaluate_expression(right, env);
        }

        // Evaluate single condition
        self.evaluate_condition(expr.trim(), env)
    }

    fn evaluate_condition(&self, cond: &str, env: &Environment) -> bool {
        let cond = cond.trim();

        // Remove parentheses
        let cond = if cond.starts_with('(') && cond.ends_with(')') {
            &cond[1..cond.len() - 1]
        } else {
            cond
        };

        // Parse condition
        let (op, op_str) = if cond.contains("!=") {
            (MarkerOp::NotEq, "!=")
        } else if cond.contains("==") {
            (MarkerOp::Eq, "==")
        } else if cond.contains("<=") {
            (MarkerOp::LtEq, "<=")
        } else if cond.contains(">=") {
            (MarkerOp::GtEq, ">=")
        } else if cond.contains('<') {
            (MarkerOp::Lt, "<")
        } else if cond.contains('>') {
            (MarkerOp::Gt, ">")
        } else if cond.contains(" in ") {
            (MarkerOp::In, " in ")
        } else if cond.contains(" not in ") {
            (MarkerOp::NotIn, " not in ")
        } else {
            return false;
        };

        let parts: Vec<&str> = cond.splitn(2, op_str).collect();
        if parts.len() != 2 {
            return false;
        }

        let variable = parts[0].trim().trim_matches('\'').trim_matches('"');
        let value = parts[1].trim().trim_matches('\'').trim_matches('"');

        let var_value = self.get_variable_value(variable, env);

        match op {
            MarkerOp::Eq => var_value == value,
            MarkerOp::NotEq => var_value != value,
            MarkerOp::Lt => self.compare_versions(&var_value, value) < 0,
            MarkerOp::LtEq => self.compare_versions(&var_value, value) <= 0,
            MarkerOp::Gt => self.compare_versions(&var_value, value) > 0,
            MarkerOp::GtEq => self.compare_versions(&var_value, value) >= 0,
            MarkerOp::In => value.contains(&var_value),
            MarkerOp::NotIn => !value.contains(&var_value),
        }
    }

    fn get_variable_value(&self, var: &str, env: &Environment) -> String {
        match var {
            "python_version" => env.python_version.clone(),
            "python_full_version" => env.python_full_version.clone(),
            "os_name" => env.os_name.clone(),
            "sys_platform" => env.sys_platform.clone(),
            "platform_release" => env.platform_release.clone(),
            "platform_system" => env.platform_system.clone(),
            "platform_version" => env.platform_version.clone(),
            "platform_machine" => env.platform_machine.clone(),
            "platform_python_implementation" => env.platform_python_implementation.clone(),
            "implementation_name" => env.implementation_name.clone(),
            "implementation_version" => env.implementation_version.clone(),
            _ => String::new(),
        }
    }

    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let v2_parts: Vec<u32> = v2
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..v1_parts.len().max(v2_parts.len()) {
            let p1 = v1_parts.get(i).copied().unwrap_or(0);
            let p2 = v2_parts.get(i).copied().unwrap_or(0);
            if p1 < p2 {
                return -1;
            } else if p1 > p2 {
                return 1;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_marker() {
        let marker = Marker::parse("python_version >= '3.6'").unwrap();
        assert_eq!(marker.expression, "python_version >= '3.6'");
    }

    #[test]
    fn test_evaluate_python_version() {
        let marker = Marker::parse("python_version >= '3.6'").unwrap();
        let env = Environment::current();
        assert!(marker.evaluate(&env));
    }

    #[test]
    fn test_evaluate_sys_platform() {
        let marker = Marker::parse("sys_platform == 'darwin'").unwrap();
        let mut env = Environment::current();
        env.sys_platform = "darwin".to_string();
        assert!(marker.evaluate(&env));
    }

    #[test]
    fn test_evaluate_and_condition() {
        let marker = Marker::parse("python_version >= '3.6' and sys_platform == 'darwin'").unwrap();
        let mut env = Environment::current();
        env.sys_platform = "darwin".to_string();
        assert!(marker.evaluate(&env));
    }

    #[test]
    fn test_evaluate_or_condition() {
        let marker = Marker::parse("sys_platform == 'win32' or sys_platform == 'darwin'").unwrap();
        let mut env = Environment::current();
        env.sys_platform = "darwin".to_string();
        assert!(marker.evaluate(&env));
    }
}
