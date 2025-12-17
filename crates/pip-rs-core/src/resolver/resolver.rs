/// Dependency resolution algorithm
use crate::models::{Package, Requirement, VersionOp, Marker, Environment};
use anyhow::Result;
use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct Resolver {
    cache: HashMap<String, Package>,
    visited: HashSet<String>,
    environment: Environment,
    constraints: HashMap<String, Vec<Requirement>>,
    version_cache: HashMap<String, Vec<u32>>, // Cache parsed version parts
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            visited: HashSet::new(),
            environment: Environment::current(),
            constraints: HashMap::new(),
            version_cache: HashMap::new(),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        Self {
            cache: HashMap::new(),
            visited: HashSet::new(),
            environment,
            constraints: HashMap::new(),
            version_cache: HashMap::new(),
        }
    }

    /// Set constraints from a constraints file
    pub fn set_constraints(&mut self, constraints: Vec<Requirement>) {
        for req in constraints {
            self.constraints
                .entry(req.name.clone())
                .or_insert_with(Vec::new)
                .push(req);
        }
    }

    pub async fn resolve(&mut self, requirements: Vec<Requirement>) -> Result<Vec<Package>> {
        // Use concurrent resolution for better performance
        self.resolve_concurrent(requirements, 10).await
    }

    /// Resolve dependencies with bounded concurrency for better performance
    async fn resolve_concurrent(&mut self, requirements: Vec<Requirement>, max_concurrent: usize) -> Result<Vec<Package>> {
        use futures::future;
        
        let mut resolved = Vec::new();
        let mut queue: VecDeque<Requirement> = requirements.into_iter().collect();
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        while !queue.is_empty() {
            // Collect batch of requirements to fetch concurrently
            let mut batch = Vec::new();
            while batch.len() < max_concurrent && !queue.is_empty() {
                if let Some(req) = queue.pop_front() {
                    if self.visited.contains(&req.name) {
                        continue;
                    }
                    self.visited.insert(req.name.clone());
                    batch.push(req);
                }
            }

            if batch.is_empty() {
                break;
            }

            // Fetch all packages in batch concurrently
            let sem = semaphore.clone();
            let batch_clone: Vec<_> = batch.iter().map(|req| {
                (req.name.clone(), req.specs.clone(), self.constraints.get(&req.name).cloned())
            }).collect();
            
            let handles: Vec<_> = batch_clone.into_iter().zip(batch.iter()).map(|((name, specs, constraint_reqs), req)| {
                let sem = sem.clone();
                let req_name = req.name.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire().await;
                    let result = crate::network::get_package_metadata(&name, "latest").await;
                    (req_name, result, specs, constraint_reqs)
                })
            }).collect();

            // Wait for all in batch to complete
            let results = future::join_all(handles).await;

            // Process results
            for result in results {
                match result {
                    Ok((req_name, package_result, specs, constraint_reqs)) => {
                        match package_result {
                            Ok(package) => {
                                // Check version constraints from requirement
                                if !self.satisfies_version(&package.version, &specs) {
                                    tracing::warn!(
                                        "Package {} version {} does not satisfy requirements",
                                        package.name,
                                        package.version
                                    );
                                    continue;
                                }
                                
                                // Check constraints file constraints
                                if let Some(constraint_reqs) = constraint_reqs {
                                    let mut satisfies_constraints = true;
                                    for constraint_req in &constraint_reqs {
                                        if !self.satisfies_version(&package.version, &constraint_req.specs) {
                                            satisfies_constraints = false;
                                            break;
                                        }
                                    }
                                    if !satisfies_constraints {
                                        continue;
                                    }
                                }
                                
                                // Cache the package
                                self.cache.insert(package.name.clone(), package.clone());
                                
                                // Package satisfies all constraints
                                // Parse dependencies, filtering by environment markers
                                for dep_str in &package.requires_dist {
                                    if let Ok(dep_req) = dep_str.parse::<Requirement>() {
                                        // Check if dependency applies to current environment
                                        if let Some(marker_str) = &dep_req.marker {
                                            if let Ok(marker) = Marker::parse(marker_str) {
                                                if !marker.evaluate(&self.environment) {
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
                            }
                            Err(e) => {
                                tracing::warn!("Failed to fetch package {}: {}", req_name, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Task error: {}", e);
                    }
                }
            }
        }

        Ok(resolved)
    }

    /// Legacy sequential resolution (kept for compatibility)
    #[allow(dead_code)]
    pub async fn resolve_sequential(&mut self, requirements: Vec<Requirement>) -> Result<Vec<Package>> {
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
                    // Check version constraints from requirement
                    if !self.satisfies_version(&package.version, &req.specs) {
                        tracing::warn!(
                            "Package {} version {} does not satisfy requirements",
                            package.name,
                            package.version
                        );
                        continue;
                    }
                    
                    // Check constraints file constraints
                    if let Some(constraint_reqs) = self.constraints.get(&req.name) {
                        let mut satisfies_constraints = true;
                        let constraint_reqs_clone = constraint_reqs.clone();
                        for constraint_req in constraint_reqs_clone {
                            if !self.satisfies_version(&package.version, &constraint_req.specs) {
                                tracing::warn!(
                                    "Package {} version {} does not satisfy constraint {}",
                                    package.name,
                                    package.version,
                                    constraint_req.specs.iter()
                                        .map(|s| format!("{:?} {}", s.op, s.version))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                );
                                satisfies_constraints = false;
                                break;
                            }
                        }
                        if !satisfies_constraints {
                            continue;
                        }
                    }
                    
                    // Package satisfies all constraints
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

    fn satisfies_version(&mut self, version: &str, specs: &[crate::models::VersionSpec]) -> bool {
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

    fn check_version_spec(&mut self, version: &str, spec: &crate::models::VersionSpec) -> bool {
        use std::cmp::Ordering;

        // Use cached parsed versions to avoid repeated parsing
        let v1_parts = self.parse_version_cached(version);
        let v2_parts = self.parse_version_cached(&spec.version);

        let mut cmp = Ordering::Equal;
        for i in 0..v1_parts.len().max(v2_parts.len()) {
            let v1 = v1_parts.get(i).copied().unwrap_or(0);
            let v2 = v2_parts.get(i).copied().unwrap_or(0);
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
                // Check major.minor match
                let v1_major = v1_parts.get(0).copied().unwrap_or(0);
                let v1_minor = v1_parts.get(1).copied().unwrap_or(0);
                let v2_major = v2_parts.get(0).copied().unwrap_or(0);
                let v2_minor = v2_parts.get(1).copied().unwrap_or(0);
                v1_major == v2_major && v1_minor == v2_minor && cmp != Ordering::Less
            }
        }
    }

    /// Parse version and cache the result to avoid repeated parsing
    fn parse_version_cached(&mut self, version: &str) -> Vec<u32> {
        // Check cache first
        if let Some(cached) = self.version_cache.get(version) {
            return cached.clone();
        }
        
        // Parse and cache
        let parts: Vec<u32> = version
            .split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect();
        
        // Cache the parsed version
        self.version_cache.insert(version.to_string(), parts.clone());
        parts
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
        let mut resolver = Resolver::new();
        
        let spec = crate::models::VersionSpec {
            op: VersionOp::GtEq,
            version: "2.0.0".to_string(),
        };
        assert!(resolver.check_version_spec("2.1.0", &spec));
        assert!(!resolver.check_version_spec("1.9.0", &spec));
    }
}

