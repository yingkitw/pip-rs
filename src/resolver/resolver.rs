/// Dependency resolution algorithm
use crate::models::{Package, Requirement, VersionOp, Marker, Environment};
use anyhow::Result;
use std::collections::{HashMap, VecDeque, HashSet};

pub struct Resolver {
    cache: HashMap<String, Package>,
    visited: HashSet<String>,
    environment: Environment,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            visited: HashSet::new(),
            environment: Environment::current(),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        Self {
            cache: HashMap::new(),
            visited: HashSet::new(),
            environment,
        }
    }

    pub async fn resolve(&mut self, requirements: Vec<Requirement>) -> Result<Vec<Package>> {
        let mut resolved = Vec::new();
        let mut queue: VecDeque<Requirement> = requirements.into_iter().collect();

        while let Some(req) = queue.pop_front() {
            if self.visited.contains(&req.name) {
                continue;
            }
            self.visited.insert(req.name.clone());

            // Get package metadata
            match self.get_package(&req.name).await {
                Ok(package) => {
                    // Check version constraints
                    if self.satisfies_version(&package.version, &req.specs) {
                        // Parse dependencies, filtering by environment markers
                        for dep_str in &package.requires_dist {
                            if let Ok(dep_req) = dep_str.parse::<Requirement>() {
                                // Check if dependency applies to current environment
                                if let Some(marker_str) = &dep_req.marker {
                                    if let Ok(marker) = Marker::parse(marker_str) {
                                        if !marker.evaluate(&self.environment) {
                                            // Skip this dependency - doesn't apply to current environment
                                            continue;
                                        }
                                    }
                                }

                                if !self.visited.contains(&dep_req.name) {
                                    queue.push_back(dep_req);
                                }
                            }
                        }
                        resolved.push(package);
                    } else {
                        tracing::warn!(
                            "Package {} version {} does not satisfy requirements",
                            package.name,
                            package.version
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch package {}: {}", req.name, e);
                }
            }
        }

        Ok(resolved)
    }

    async fn get_package(&mut self, name: &str) -> Result<Package> {
        if let Some(pkg) = self.cache.get(name) {
            return Ok(pkg.clone());
        }

        let package = crate::network::get_package_metadata(name, "latest").await?;
        self.cache.insert(name.to_string(), package.clone());
        Ok(package)
    }

    fn satisfies_version(&self, version: &str, specs: &[crate::models::VersionSpec]) -> bool {
        if specs.is_empty() {
            return true;
        }

        for spec in specs {
            if !self.check_version_spec(version, spec) {
                return false;
            }
        }
        true
    }

    fn check_version_spec(&self, version: &str, spec: &crate::models::VersionSpec) -> bool {
        use std::cmp::Ordering;

        let v1_parts: Vec<&str> = version.split('.').collect();
        let v2_parts: Vec<&str> = spec.version.split('.').collect();

        let mut cmp = Ordering::Equal;
        for i in 0..v1_parts.len().max(v2_parts.len()) {
            let v1 = v1_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            let v2 = v2_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            cmp = v1.cmp(&v2);
            if cmp != Ordering::Equal {
                break;
            }
        }

        match spec.op {
            VersionOp::Eq => cmp == Ordering::Equal,
            VersionOp::NotEq => cmp != Ordering::Equal,
            VersionOp::Lt => cmp == Ordering::Less,
            VersionOp::LtEq => cmp != Ordering::Greater,
            VersionOp::Gt => cmp == Ordering::Greater,
            VersionOp::GtEq => cmp != Ordering::Less,
            VersionOp::Compatible => {
                // ~= compatible release: allows patch-level changes
                let v1_major_minor = format!("{}.{}", 
                    v1_parts.get(0).unwrap_or(&"0"),
                    v1_parts.get(1).unwrap_or(&"0")
                );
                let v2_major_minor = format!("{}.{}", 
                    v2_parts.get(0).unwrap_or(&"0"),
                    v2_parts.get(1).unwrap_or(&"0")
                );
                v1_major_minor == v2_major_minor && cmp != Ordering::Less
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.visited.clear();
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let resolver = Resolver::new();
        
        let spec = crate::models::VersionSpec {
            op: VersionOp::GtEq,
            version: "2.0.0".to_string(),
        };
        assert!(resolver.check_version_spec("2.1.0", &spec));
        assert!(!resolver.check_version_spec("1.9.0", &spec));
    }
}
